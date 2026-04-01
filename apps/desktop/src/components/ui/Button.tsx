import { Slot } from "@radix-ui/react-slot";
import type { ButtonHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

type ButtonVariant = "default" | "secondary" | "ghost";
type ButtonSize = "default" | "sm" | "icon";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  asChild?: boolean;
  variant?: ButtonVariant;
  size?: ButtonSize;
}

const variantClasses: Record<ButtonVariant, string> = {
  default:
    "bg-[var(--color-brand)] text-white hover:brightness-105 active:brightness-95",
  secondary:
    "bg-[var(--color-surface-muted)] text-[var(--color-text)] hover:bg-white/10",
  ghost:
    "bg-transparent text-[var(--color-text-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-text)]",
};

const sizeClasses: Record<ButtonSize, string> = {
  default: "h-11 px-4 py-2.5",
  sm: "h-9 px-3.5 py-2",
  icon: "h-11 w-11",
};

export function Button({
  asChild = false,
  className,
  size = "default",
  variant = "default",
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      className={cn(
        "inline-flex cursor-pointer items-center justify-center rounded-[1rem] border border-transparent text-sm font-medium transition-[background-color,border-color,color,transform,filter] active:scale-[0.99] disabled:cursor-not-allowed disabled:pointer-events-none disabled:opacity-50",
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
      {...props}
    />
  );
}
