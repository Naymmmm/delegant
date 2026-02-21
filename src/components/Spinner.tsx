import { clsx } from "clsx";

export function Spinner({ size = 16, className }: { size?: number; className?: string }) {
  return (
    <div
      className={clsx("rounded-full border-2 border-current border-t-transparent animate-spinner", className)}
      style={{ width: size, height: size }}
    />
  );
}
