import * as ContextMenuPrimitive from "@kobalte/core/context-menu";
import type { PolymorphicProps } from "@kobalte/core/polymorphic";
import type { ValidComponent } from "solid-js";
import { splitProps } from "solid-js";

import { cn } from "~/lib/utils";

const ContextMenu = ContextMenuPrimitive.Root;
const ContextMenuTrigger = ContextMenuPrimitive.Trigger;
const ContextMenuSub = ContextMenuPrimitive.Sub;

const contentClass =
  "z-50 min-w-36 rounded-md border bg-popover p-1 text-popover-foreground shadow-md outline-none";
const itemClass =
  "relative flex w-full cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50";

type ContextMenuContentProps<T extends ValidComponent = "div"> =
  ContextMenuPrimitive.ContextMenuContentProps<T> & {
    class?: string | undefined;
  };

const ContextMenuContent = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, ContextMenuContentProps<T>>,
) => {
  const [local, others] = splitProps(props as ContextMenuContentProps, [
    "class",
  ]);
  return (
    <ContextMenuPrimitive.Portal>
      <ContextMenuPrimitive.Content
        class={cn(contentClass, local.class)}
        {...others}
      />
    </ContextMenuPrimitive.Portal>
  );
};

type ContextMenuSubContentProps<T extends ValidComponent = "div"> =
  ContextMenuPrimitive.ContextMenuSubContentProps<T> & {
    class?: string | undefined;
  };

const ContextMenuSubContent = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, ContextMenuSubContentProps<T>>,
) => {
  const [local, others] = splitProps(props as ContextMenuSubContentProps, [
    "class",
  ]);
  return (
    <ContextMenuPrimitive.Portal>
      <ContextMenuPrimitive.SubContent
        class={cn(contentClass, local.class)}
        {...others}
      />
    </ContextMenuPrimitive.Portal>
  );
};

type ContextMenuItemProps<T extends ValidComponent = "div"> =
  ContextMenuPrimitive.ContextMenuItemProps<T> & {
    class?: string | undefined;
  };

const ContextMenuItem = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, ContextMenuItemProps<T>>,
) => {
  const [local, others] = splitProps(props as ContextMenuItemProps, ["class"]);
  return (
    <ContextMenuPrimitive.Item
      class={cn(itemClass, local.class)}
      {...others}
    />
  );
};

type ContextMenuSubTriggerProps<T extends ValidComponent = "div"> =
  ContextMenuPrimitive.ContextMenuSubTriggerProps<T> & {
    class?: string | undefined;
  };

const ContextMenuSubTrigger = <T extends ValidComponent = "div">(
  props: PolymorphicProps<T, ContextMenuSubTriggerProps<T>>,
) => {
  const [local, others] = splitProps(props as ContextMenuSubTriggerProps, [
    "class",
  ]);
  return (
    <ContextMenuPrimitive.SubTrigger
      class={cn(itemClass, "justify-between", local.class)}
      {...others}
    />
  );
};

export {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
};
