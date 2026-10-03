import { EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { api, type Note, unwrap } from "~/api/client";

export type SaveStatus = "saved" | "dirty" | "saving" | "conflict" | "error";

const AUTOSAVE_DELAY_MS = 1000;
const RETRY_DELAY_MS = 5000;

interface Entry {
  state: EditorState;
  /** Server version the saved fields were last in sync with. */
  version: string;
  savedBody: string;
  tags: string[];
  savedTags: string[];
  conflict: boolean;
  failed: boolean;
  saving: boolean;
}

interface Hooks {
  onStatus: (status: SaveStatus) => void;
  onTags: (tags: string[]) => void;
  onSaved: (note: Note) => void;
  onError: (message: string) => void;
}

interface SaveOptions {
  force?: boolean;
  keepalive?: boolean;
}

function sameTags(a: string[], b: string[]) {
  return a.length === b.length && a.every((t, i) => t === b[i]);
}

/**
 * Owns the single EditorView and one cached EditorState per note, so each
 * note keeps its own undo history. It lives outside Solid's reactivity; the
 * component only receives status and tags through hooks.
 */
export class EditorSession {
  private view: EditorView;
  private extensions: Extension[];
  private hooks: Hooks;
  private entries = new Map<string, Entry>();
  private currentId: string | undefined;
  private timer: ReturnType<typeof setTimeout> | undefined;

  constructor(parent: HTMLElement, extensions: Extension[], hooks: Hooks) {
    this.hooks = hooks;
    this.extensions = [
      ...extensions,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) this.touched();
      }),
      EditorView.domEventHandlers({
        blur: () => {
          void this.flush();
        },
      }),
    ];
    this.view = new EditorView({ parent, state: this.createState("") });
  }

  get editorView() {
    return this.view;
  }

  open(note: Note) {
    const id = note.meta.id;
    this.stash();
    let entry = this.entries.get(id);
    if (entry && entry.version !== note.version && !this.isDirty(entry)) {
      entry = undefined;
    }
    if (!entry) {
      entry = {
        state: this.createState(note.body),
        version: note.version,
        savedBody: note.body,
        tags: [...note.meta.tags],
        savedTags: [...note.meta.tags],
        conflict: false,
        failed: false,
        saving: false,
      };
      this.entries.set(id, entry);
    }
    if (this.currentId && this.currentId !== id) void this.save(this.currentId);
    this.currentId = id;
    if (this.view.state !== entry.state) {
      this.view.setState(entry.state);
      if (entry.state.doc.length === 0) this.view.focus();
    }
    this.publish(entry);
  }

  /** No note is selected. */
  close() {
    this.stash();
    if (this.currentId) void this.save(this.currentId);
    this.currentId = undefined;
    this.view.setState(this.createState(""));
  }

  /** Drops local state for a note that no longer exists. */
  forget(id: string) {
    this.entries.delete(id);
    if (this.currentId === id) {
      clearTimeout(this.timer);
      this.currentId = undefined;
      this.view.setState(this.createState(""));
    }
  }

  setTags(tags: string[]) {
    const entry = this.current();
    if (!entry) return;
    entry.tags = tags;
    this.touched();
  }

  flush(options: SaveOptions = {}) {
    clearTimeout(this.timer);
    return this.currentId
      ? this.save(this.currentId, options)
      : Promise.resolve();
  }

  async save(
    id: string,
    { force = false, keepalive = false }: SaveOptions = {},
  ) {
    const entry = this.entries.get(id);
    if (!entry || entry.saving) return;
    if (id === this.currentId) entry.state = this.view.state;
    if (entry.conflict && !force) return;

    const body = entry.state.doc.toString();
    const tags = [...entry.tags];
    if (!force && !this.isDirty(entry)) {
      this.publishIfCurrent(id, entry);
      return;
    }

    entry.saving = true;
    this.publishIfCurrent(id, entry);
    try {
      const { data, response } = await api.PUT("/api/notes/{id}", {
        params: { path: { id } },
        body: { body, tags, base_version: entry.version, force },
        keepalive,
      });
      entry.saving = false;
      if (response.status === 409) {
        entry.conflict = true;
      } else if (!response.ok || !data) {
        throw new Error(response.statusText);
      } else {
        entry.version = data.version;
        entry.savedBody = body;
        entry.savedTags = tags;
        entry.conflict = false;
        entry.failed = false;
        this.hooks.onSaved(data);
      }
    } catch (e) {
      entry.saving = false;
      entry.failed = true;
      this.hooks.onError(e instanceof Error ? e.message : String(e));
      this.schedule(id, RETRY_DELAY_MS);
    }

    this.publishIfCurrent(id, entry);
    if (!entry.conflict && !entry.failed && this.isDirty(entry)) {
      this.schedule(id, AUTOSAVE_DELAY_MS);
    }
  }

  /** Throws away local edits and loads the server's version. */
  async discard(id: string) {
    const note = await unwrap(
      api.GET("/api/notes/{id}", { params: { path: { id } } }),
    );
    this.entries.delete(id);
    this.hooks.onSaved(note);
    this.open(note);
  }

  destroy() {
    clearTimeout(this.timer);
    this.view.destroy();
  }

  private createState(doc: string) {
    return EditorState.create({ doc, extensions: this.extensions });
  }

  private current() {
    return this.currentId ? this.entries.get(this.currentId) : undefined;
  }

  /** Copies the live editor state into the current entry. */
  private stash() {
    const entry = this.current();
    if (entry) entry.state = this.view.state;
  }

  private isDirty(entry: Entry) {
    return (
      entry.state.doc.toString() !== entry.savedBody ||
      !sameTags(entry.tags, entry.savedTags)
    );
  }

  private touched() {
    const entry = this.current();
    if (!this.currentId || !entry) return;
    entry.state = this.view.state;
    entry.failed = false;
    this.publish(entry);
    this.schedule(this.currentId, AUTOSAVE_DELAY_MS);
  }

  private schedule(id: string, delay: number) {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.save(id), delay);
  }

  private statusOf(entry: Entry): SaveStatus {
    if (entry.conflict) return "conflict";
    if (entry.saving) return "saving";
    if (entry.failed) return "error";
    return this.isDirty(entry) ? "dirty" : "saved";
  }

  private publish(entry: Entry) {
    this.hooks.onTags(entry.tags);
    this.hooks.onStatus(this.statusOf(entry));
  }

  private publishIfCurrent(id: string, entry: Entry) {
    if (id === this.currentId) this.publish(entry);
  }
}
