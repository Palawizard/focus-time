import * as TooltipPrimitive from "@radix-ui/react-tooltip";

import { cn } from "../../lib/cn";

export const TooltipProvider = TooltipPrimitive.Provider;
export const Tooltip = TooltipPrimitive.Root;
export const TooltipTrigger = TooltipPrimitive.Trigger;

export function TooltipContent({
  className,
  sideOffset = 10,
  ...props
}: TooltipPrimitive.TooltipContentProps) {
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        sideOffset={sideOffset}
        className={cn(
          "z-50 rounded-[0.85rem] border border-[var(--color-border)] bg-[var(--color-surface-strong)] px-3 py-2 text-xs text-[var(--color-text)] shadow-[var(--shadow-card)]",
          className,
        )}
        {...props}
      />
    </TooltipPrimitive.Portal>
  );
}
