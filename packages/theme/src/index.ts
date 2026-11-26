/**
 * The Mikata theme for TypeScript: the CLI (Ink takes hex colours) and anything that cannot
 * read CSS. The same values live in tokens.css; the test keeps them equal.
 */

export interface Palette {
  bg: string;
  panel: string;
  panelRaised: string;
  line: string;
  text: string;
  textMuted: string;
  holo: string;
  alarm: string;
  gold: string;
}

export const dark: Palette = {
  bg: '#060b12',
  panel: '#0d141f',
  panelRaised: '#142033',
  line: '#1f3346',
  text: '#e0e6ed',
  textMuted: '#a1aab5',
  holo: '#00f2ff',
  alarm: '#ff2a7d',
  gold: '#f7d002',
};

export const light: Palette = {
  bg: '#f3f6fa',
  panel: '#ffffff',
  panelRaised: '#e9eff6',
  line: '#c9d5e2',
  text: '#0d141f',
  textMuted: '#46505c',
  holo: '#00707a',
  alarm: '#c0004c',
  gold: '#7a5c00',
};

/** One colour per race and form: the cards' frames, the board's lanes, the CLI's prefixes. */
export const races = {
  summoner: '#3fa9f5',
  elf: '#2fa36b',
  orc: '#c9822b',
  slime: '#19c3b0',
  kijin: '#d7263d',
  darkElf: '#7b4fd6',
  highElf: '#f5e6b8',
  highOrc: '#e8a83c',
  namedSlime: '#a0e8f0',
  oni: '#e8b04a',
  archDarkElf: '#c4cce4',
  player: '#d4af37',
} as const;

export type RaceKey = keyof typeof races;

/**
 * Mikata's logo colours, fixed in every scheme and mode: gold for you, cyan for Mikata, and a
 * deeper ink of each for a light page. The logo is the brand, not the interface's accent: a
 * scheme that turns the accent magenta never turns the logo.
 */
export const brand = {
  you: '#f7d002',
  ally: '#00f2ff',
  youInk: '#8a6d00',
  allyInk: '#00747e',
} as const;

/**
 * A colour scheme: the board and the terminal app wear the same ones. `hud` is the palette above
 * (`dark`, `light`); the others are built on four-colour palettes, their values derived in the
 * terminal app's `theme.rs` — a test there keeps the two equal, and prints these on a change.
 */
export interface SchemePalette extends Palette {
  /** The human's colour: their lines, the ask, the gate. */
  player: string;
}

export const SCHEMES = ['neon', 'ember', 'lagoon', 'harbor', 'dune', 'spice', 'hud'] as const;
export type SchemeName = (typeof SCHEMES)[number];
/** The scheme worn unless another is asked for. */
export const DEFAULT_SCHEME: SchemeName = 'neon';

export const neonDark: SchemePalette = {
  bg: '#07050a',
  panel: '#110c14',
  panelRaised: '#1b1420',
  line: '#3d2b40',
  text: '#f5eff7',
  textMuted: '#b7a9bc',
  holo: '#ff168a',
  alarm: '#ff5575',
  gold: '#ffb020',
  player: '#ffb020',
};

export const neonLight: SchemePalette = {
  bg: '#faf7fa',
  panel: '#ffffff',
  panelRaised: '#f3ecf3',
  line: '#e2d4e2',
  text: '#211626',
  textMuted: '#65546b',
  holo: '#c0125c',
  alarm: '#b01c42',
  gold: '#8f5506',
  player: '#8f5506',
};

export const emberDark: SchemePalette = {
  bg: '#1d1410',
  panel: '#2a1e1b',
  panelRaised: '#372b2b',
  line: '#5d4131',
  text: '#e1e5e8',
  textMuted: '#aaa19f',
  holo: '#ff9a00',
  alarm: '#ed7566',
  gold: '#f7d794',
  player: '#f7d794',
};

export const emberLight: SchemePalette = {
  bg: '#f3f3f0',
  panel: '#fffdfa',
  panelRaised: '#e9e4e2',
  line: '#d2b8aa',
  text: '#241c1c',
  textMuted: '#575253',
  holo: '#87570f',
  alarm: '#ad362a',
  gold: '#665e4b',
  player: '#665e4b',
};

export const lagoonDark: SchemePalette = {
  bg: '#09161c',
  panel: '#11212a',
  panelRaised: '#172e3d',
  line: '#224854',
  text: '#e2e7e9',
  textMuted: '#92a4ad',
  holo: '#aeeed3',
  alarm: '#ff6f61',
  gold: '#fff8b0',
  player: '#fff8b0',
};

export const lagoonLight: SchemePalette = {
  bg: '#f4f6f3',
  panel: '#fffffb',
  panelRaised: '#dae6ec',
  line: '#a1bec7',
  text: '#101e28',
  textMuted: '#43545e',
  holo: '#486663',
  alarm: '#9d4c47',
  gold: '#606351',
  player: '#606351',
};

export const harborDark: SchemePalette = {
  bg: '#080f1a',
  panel: '#0f1827',
  panelRaised: '#15243a',
  line: '#1e344d',
  text: '#dce6ed',
  textMuted: '#919caa',
  holo: '#60c1e5',
  alarm: '#ff5c7a',
  gold: '#a0e0ef',
  player: '#a0e0ef',
};

export const harborLight: SchemePalette = {
  bg: '#ebf4f9',
  panel: '#fafdfe',
  panelRaised: '#d9e0ea',
  line: '#9eadc1',
  text: '#0e1725',
  textMuted: '#424d5c',
  holo: '#34657d',
  alarm: '#9d3f56',
  gold: '#45626e',
  player: '#45626e',
};

export const duneDark: SchemePalette = {
  bg: '#111a24',
  panel: '#1a2735',
  panelRaised: '#23354b',
  line: '#38556e',
  text: '#e2e4e4',
  textMuted: '#9ba9b8',
  holo: '#ff933e',
  alarm: '#e48689',
  gold: '#ffcb56',
  player: '#ffcb56',
};

export const duneLight: SchemePalette = {
  bg: '#f4f2ea',
  panel: '#fffcf7',
  panelRaised: '#e0e9f2',
  line: '#b4c9dc',
  text: '#172330',
  textMuted: '#4b5967',
  holo: '#8c5730',
  alarm: '#a4383f',
  gold: '#695a34',
  player: '#695a34',
};

export const spiceDark: SchemePalette = {
  bg: '#0a191f',
  panel: '#12252f',
  panelRaised: '#183343',
  line: '#24515f',
  text: '#e2e2df',
  textMuted: '#93a8b2',
  holo: '#3cb8b8',
  alarm: '#e57d77',
  gold: '#ff9f00',
  player: '#ff9f00',
};

export const spiceLight: SchemePalette = {
  bg: '#f4ede1',
  panel: '#fffaf2',
  panelRaised: '#dae8ee',
  line: '#a3c6d0',
  text: '#11212b',
  textMuted: '#445762',
  holo: '#276e73',
  alarm: '#a62321',
  gold: '#694913',
  player: '#694913',
};

/** Every scheme's palettes; `hud` is Mikata's own, with the races' player colour. */
export const schemes: Record<SchemeName, { dark: SchemePalette; light: SchemePalette }> = {
  neon: { dark: neonDark, light: neonLight },
  ember: { dark: emberDark, light: emberLight },
  lagoon: { dark: lagoonDark, light: lagoonLight },
  harbor: { dark: harborDark, light: harborLight },
  dune: { dark: duneDark, light: duneLight },
  spice: { dark: spiceDark, light: spiceLight },
  hud: { dark: { ...dark, player: races.player }, light: { ...light, player: races.player } },
};

/** WCAG relative luminance of a #rrggbb colour. */
export function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const c = Number.parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two #rrggbb colours, from 1 to 21. */
export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/**
 * The board's palettes: the owner's seven gradient systems, worn over the native look
 * (`data-palette` on `<html>`); tokens.css holds the same values, a test keeps them equal.
 */
export const PALETTES = [
  'solar-flare',
  'aurora',
  'ultraviolet',
  'cryo',
  'crimson-alloy',
  'golden-jade',
  'copper-iris',
] as const;
export type PaletteName = (typeof PALETTES)[number];

/** One mode of a palette: the surfaces, the text roles, the brand's stops and the words on it. */
export interface BoardPalette extends Palette {
  hot: string;
  coral: string;
  amber: string;
  onBrand: string;
  successText: string;
  error: string;
  info: string;
  warning: string;
}

export const palettes: Record<
  PaletteName,
  { name: string; about: string; dark: BoardPalette; light: BoardPalette }
> = {
  'solar-flare': {
    name: 'Solar Flare',
    about: 'Energy · execution · performance',
    dark: {
      bg: '#100c09',
      panel: '#21170f',
      panelRaised: '#2e241c',
      line: '#5f320b',
      text: '#fff8ef',
      textMuted: '#9f9a93',
      holo: '#ff7900',
      alarm: '#ff5575',
      gold: '#ffd60a',
      hot: '#ff3d00',
      coral: '#ff7900',
      amber: '#ffd60a',
      onBrand: '#0b0806',
      successText: '#c5f768',
      error: '#ff5575',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#fff8ef',
      panel: '#fffcf9',
      panelRaised: '#f7e9d7',
      line: '#f0caa8',
      text: '#100c09',
      textMuted: '#6b6660',
      holo: '#b92d00',
      alarm: '#b01c42',
      gold: '#896204',
      hot: '#c75733',
      coral: '#d95b00',
      amber: '#d99b00',
      onBrand: '#0b0806',
      successText: '#587121',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  aurora: {
    name: 'Aurora',
    about: 'Autonomy · nature · adaptive systems',
    dark: {
      bg: '#08120f',
      panel: '#12241b',
      panelRaised: '#203128',
      line: '#245931',
      text: '#f4fbf2',
      textMuted: '#969e97',
      holo: '#52e06b',
      alarm: '#ff5c7a',
      gold: '#d9f95c',
      hot: '#00c896',
      coral: '#52e06b',
      amber: '#d9f95c',
      onBrand: '#060d0a',
      successText: '#e1fa7d',
      error: '#ff5c7a',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#f4fbf2',
      panel: '#fbfdfa',
      panelRaised: '#e5f3e2',
      line: '#bbdcbd',
      text: '#08120f',
      textMuted: '#626b65',
      holo: '#017a61',
      alarm: '#b01c42',
      gold: '#597307',
      hot: '#088b6f',
      coral: '#2b8c3d',
      amber: '#a0c900',
      onBrand: '#060d0a',
      successText: '#5e712e',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  ultraviolet: {
    name: 'Ultraviolet',
    about: 'Research · intelligence · experimentation',
    dark: {
      bg: '#100b1a',
      panel: '#20152f',
      panelRaised: '#2d223b',
      line: '#492969',
      text: '#faf5ff',
      textMuted: '#9c97a3',
      holo: '#ba68ff',
      alarm: '#ff5575',
      gold: '#f472d0',
      hot: '#885aff',
      coral: '#b45cff',
      amber: '#f472d0',
      onBrand: '#0b0812',
      successText: '#c5f768',
      error: '#ff5575',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#faf5ff',
      panel: '#fdfbff',
      panelRaised: '#eee4f9',
      line: '#dabeee',
      text: '#100b1a',
      textMuted: '#696471',
      holo: '#5925d0',
      alarm: '#b01c42',
      gold: '#b13382',
      hot: '#5925d0',
      coral: '#9139c8',
      amber: '#cb3993',
      onBrand: '#ffffff',
      successText: '#566e2b',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  cryo: {
    name: 'Cryo',
    about: 'Precision · observability · security',
    dark: {
      bg: '#08131c',
      panel: '#102532',
      panelRaised: '#1e323e',
      line: '#0c556b',
      text: '#f1fafc',
      textMuted: '#949ea2',
      holo: '#00cfff',
      alarm: '#fe6280',
      gold: '#9cfbff',
      hot: '#007cf0',
      coral: '#00cfff',
      amber: '#9cfbff',
      onBrand: '#060d14',
      successText: '#b5eb5d',
      error: '#fe6280',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#f1fafc',
      panel: '#f9fdfe',
      panelRaised: '#ddeff4',
      line: '#acdae4',
      text: '#08131c',
      textMuted: '#616b71',
      holo: '#005dc4',
      alarm: '#b01c42',
      gold: '#247380',
      hot: '#337dd0',
      coral: '#008ead',
      amber: '#38b9c9',
      onBrand: '#060d14',
      successText: '#4f7328',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  'crimson-alloy': {
    name: 'Crimson Alloy',
    about: 'Security · authority · reliability',
    dark: {
      bg: '#170c10',
      panel: '#28151b',
      panelRaised: '#352228',
      line: '#64262a',
      text: '#fff5f4',
      textMuted: '#a29899',
      holo: '#ff5252',
      alarm: '#ff5575',
      gold: '#ffab91',
      hot: '#dc4061',
      coral: '#ff5252',
      amber: '#ffab91',
      onBrand: '#10080b',
      successText: '#c5f768',
      error: '#ff5575',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#fff5f4',
      panel: '#fffbfb',
      panelRaised: '#f6e2e0',
      line: '#efbbbd',
      text: '#170c10',
      textMuted: '#6d6365',
      holo: '#a70e30',
      alarm: '#b01c42',
      gold: '#975244',
      hot: '#a70e30',
      coral: '#d43240',
      amber: '#ae5e4c',
      onBrand: '#ffffff',
      successText: '#596d25',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  'golden-jade': {
    name: 'Golden Jade',
    about: 'Craft · sovereignty · premium tooling',
    dark: {
      bg: '#0b1512',
      panel: '#172820',
      panelRaised: '#25352d',
      line: '#28503e',
      text: '#fafaf1',
      textMuted: '#9a9e98',
      holo: '#54b78a',
      alarm: '#fe6783',
      gold: '#ebcb66',
      hot: '#008f80',
      coral: '#54b78a',
      amber: '#ebcb66',
      onBrand: '#080f0d',
      successText: '#d1f284',
      error: '#fe6783',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#fafaf1',
      panel: '#fdfdf9',
      panelRaised: '#e9efe0',
      line: '#c1d8c3',
      text: '#0b1512',
      textMuted: '#656b66',
      holo: '#007064',
      alarm: '#b01c42',
      gold: '#7e671d',
      hot: '#2e8a80',
      coral: '#3b8a62',
      amber: '#b08a22',
      onBrand: '#080f0d',
      successText: '#5a7135',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
  'copper-iris': {
    name: 'Copper Iris',
    about: 'Creativity · engineering · craftsmanship',
    dark: {
      bg: '#151015',
      panel: '#28202a',
      panelRaised: '#352d36',
      line: '#58424e',
      text: '#fcf6f2',
      textMuted: '#a09a9a',
      holo: '#d49aa9',
      alarm: '#ff627f',
      gold: '#9b8cff',
      hot: '#d97742',
      coral: '#d49aa9',
      amber: '#9b8cff',
      onBrand: '#0f0b0f',
      successText: '#c5f768',
      error: '#ff627f',
      info: '#82b1ff',
      warning: '#ffc857',
    },
    light: {
      bg: '#fcf6f2',
      panel: '#fefbfa',
      panelRaised: '#f0e4e8',
      line: '#e1c5cf',
      text: '#151015',
      textMuted: '#6b6567',
      holo: '#a64b24',
      alarm: '#b01c42',
      gold: '#6855c4',
      hot: '#a74c24',
      coral: '#aa5978',
      amber: '#6956c6',
      onBrand: '#ffffff',
      successText: '#586e28',
      error: '#b01c42',
      info: '#285fcc',
      warning: '#8f5506',
    },
  },
};

/** The native look's brand and four of its colours, to draw it beside the palettes (as
 * tokens.css's neon blocks have them). */
export const nativeLook: Record<
  'dark' | 'light',
  Pick<BoardPalette, 'hot' | 'coral' | 'amber' | 'onBrand' | 'bg' | 'panel' | 'holo' | 'gold'>
> = {
  dark: {
    hot: '#ff168a',
    coral: '#ff655c',
    amber: '#ffb020',
    onBrand: '#0d0b10',
    bg: '#07050a',
    panel: '#110c14',
    holo: '#ff168a',
    gold: '#ffb020',
  },
  light: {
    hot: '#d91670',
    coral: '#d5305a',
    amber: '#c2410c',
    onBrand: '#ffffff',
    bg: '#faf7fa',
    panel: '#ffffff',
    holo: '#c0125c',
    gold: '#8f5506',
  },
};
