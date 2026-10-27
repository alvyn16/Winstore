export interface AppPreview { src: string; caption: string }
export interface AppMedia { website?: string; previews: AppPreview[] }

const websites: Record<string, string> = {
  'Microsoft.VisualStudioCode': 'https://code.visualstudio.com/',
  'Git.Git': 'https://git-scm.com/',
  'Rustlang.Rustup': 'https://rustup.rs/',
  'OpenJS.NodeJS': 'https://nodejs.org/',
  'Valve.Steam': 'https://store.steampowered.com/',
  'HeroicGamesLauncher.HeroicGamesLauncher': 'https://heroicgameslauncher.com/',
  'PrismLauncher.PrismLauncher': 'https://prismlauncher.org/',
  'DolphinEmulator.Dolphin': 'https://dolphin-emu.org/',
  'VideoLAN.VLC': 'https://www.videolan.org/vlc/',
  'OBSProject.OBSStudio': 'https://obsproject.com/',
  'Spotify.Spotify': 'https://www.spotify.com/',
  'Discord.Discord': 'https://discord.com/',
  'Mozilla.Firefox': 'https://www.mozilla.org/firefox/',
  '7zip.7zip': 'https://www.7-zip.org/',
  'Microsoft.PowerToys': 'https://github.com/microsoft/PowerToys',
};

const previews: Record<string, AppPreview[]> = {
  'Microsoft.PowerToys': [
    { src: '/previews/powertoys-fancyzones.png', caption: 'PowerToys — FancyZones window layouts' },
    { src: '/previews/powertoys-colorpicker.png', caption: 'PowerToys — Color Picker' },
    { src: '/previews/powertoys-commandpalette.png', caption: 'PowerToys — Command Palette' },
  ],
  'Microsoft.VisualStudioCode': [
    { src: '/previews/vscode-interface.png', caption: 'Visual Studio Code — editor and workspace' },
    { src: '/previews/vscode-debugging.png', caption: 'Visual Studio Code — debugging tools' },
  ],
};

export function appMedia(id: string, website?: string, screenshots?: AppPreview[]): AppMedia {
  return { website: website ?? websites[id], previews: screenshots?.length ? screenshots : previews[id] ?? [] };
}
