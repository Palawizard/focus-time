import { LaptopMinimal, MoonStar, SunMedium } from "lucide-react";

import { Button } from "./ui/Button";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/Tooltip";
import { useThemeStore, type ThemeMode } from "../stores/theme-store";

const themeOptions: Array<{
  icon: typeof SunMedium;
  label: string;
  mode: ThemeMode;
}> = [
  { mode: "light", label: "Light", icon: SunMedium },
  { mode: "dark", label: "Dark", icon: MoonStar },
  { mode: "system", label: "System", icon: LaptopMinimal },
];

export function ThemeSwitch() {
  const mode = useThemeStore((state) => state.mode);
  const setMode = useThemeStore((state) => state.setMode);

  return (
    <div className="ft-panel-muted flex items-center gap-1 p-1">
      {themeOptions.map((option) => {
        const Icon = option.icon;

        return (
          <Tooltip key={option.mode}>
            <TooltipTrigger asChild>
              <Button
                aria-label={`Theme ${option.label}`}
                className="w-auto px-3"
                onClick={() => setMode(option.mode)}
                size="sm"
                variant={mode === option.mode ? "secondary" : "ghost"}
              >
                <Icon className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{option.label}</TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
