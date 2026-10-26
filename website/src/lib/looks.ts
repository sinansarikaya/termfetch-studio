import { defaults, groups } from "./studio";
import type { Settings } from "./studio";
export type Look = {
  id: string;
  name: string;
  description: string;
  theme: string;
  preset: string;
  logo: string;
};
export const LOOKS: Look[] = [
  {
    id: "neon",
    name: "Neon, fully loaded.",
    description: "Synthwave ’84 · Full Info · Pikachu",
    theme: "synthwave",
    preset: "full",
    logo: "pikachu",
  },
  {
    id: "developer",
    name: "A calmer workspace.",
    description: "Catppuccin Mocha · Developer · Arch",
    theme: "catppuccin-mocha",
    preset: "developer",
    logo: "arch",
  },
  {
    id: "gaming",
    name: "Built for your battlestation.",
    description: "Tokyo Night · Gaming · Master Chief",
    theme: "tokyo-night",
    preset: "gaming",
    logo: "master-chief",
  },
  {
    id: "minimal",
    name: "Just the essentials.",
    description: "Nord Aurora · Minimal · Cat",
    theme: "nord",
    preset: "minimal",
    logo: "cat",
  },
];
export function settingsForLook(look: Look): Settings {
  return {
    ...structuredClone(defaults),
    theme: look.theme,
    preset: look.preset,
    logo: look.logo,
    group_colors: Object.fromEntries(
      groups.map((group) => [group, "theme"]),
    ) as Settings["group_colors"],
  };
}
