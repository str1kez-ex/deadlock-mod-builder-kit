// constants/heroes.ts — Full 2026 Deadlock hero roster (v1.5)
// Synchronized with Rust: src-tauri/src/steam.rs::DEADLOCK_HEROES

export interface HeroEntry {
  name: string;
  icon: string;
  update?: string; // Which update introduced this hero
  role: 'Carry' | 'Brawler' | 'Initiator' | 'Mage' | 'Support';
  isGlobal?: boolean;
}

export const DEADLOCK_HEROES: HeroEntry[] = [
  { name: 'Abrams',        icon: '/heroes/bull_sm_psd.png',               role: 'Brawler' },
  { name: 'Apollo',        icon: '/heroes/fencer_sm_psd.png',             role: 'Carry',      update: 'Old Gods New Blood' },
  { name: 'Baba',          icon: '/heroes/baba_sm_psd.png',               role: 'Support',    update: 'City Never Sleeps' },
  { name: 'Bebop',         icon: '/heroes/bebop_sm_psd.png',              role: 'Initiator' },
  { name: 'Billy',         icon: '/heroes/punkgoat_sm_psd.png',           role: 'Brawler' },
  { name: 'Calico',        icon: '/heroes/nano_sm_psd.png',               role: 'Carry' },
  { name: 'Celeste',       icon: '/heroes/unicorn_sm_psd.png',            role: 'Mage',       update: 'Old Gods New Blood' },
  { name: 'Deadman Danny', icon: '/heroes/deadpack_sm_psd.png',           role: 'Brawler',    update: 'City Never Sleeps' },
  { name: 'Drifter',       icon: '/heroes/drifter_sm_psd_d94dcb01.png',   role: 'Carry' },
  { name: 'Dynamo',        icon: '/heroes/sumo_sm_psd.png',               role: 'Support' },
  { name: 'Graves',        icon: '/heroes/necro_sm_psd.png',              role: 'Brawler',    update: 'Old Gods New Blood' },
  { name: 'Grey Talon',    icon: '/heroes/archer_sm_psd.png',             role: 'Carry' },
  { name: 'Haze',          icon: '/heroes/haze_card_psd.png',             role: 'Carry' },
  { name: 'Holliday',      icon: '/heroes/astro_sm_psd.png',              role: 'Carry' },
  { name: 'Infernus',      icon: '/heroes/inferno_sm_psd.png',            role: 'Carry' },
  { name: 'Ivy',           icon: '/heroes/tengu_sm_psd.png',              role: 'Support' },
  { name: 'Kelvin',        icon: '/heroes/kelvin_sm_psd.png',             role: 'Support' },
  { name: 'Lady Geist',    icon: '/heroes/spectre_sm_psd.png',            role: 'Mage' },
  { name: 'Lash',          icon: '/heroes/lash_sm_psd.png',               role: 'Initiator' },
  { name: 'McGinnis',      icon: '/heroes/engineer_sm_psd.png',           role: 'Brawler' },
  { name: 'Mina',          icon: '/heroes/vampirebat_sm_psd.png',         role: 'Mage' },
  { name: 'Mirage',        icon: '/heroes/mirage_sm_psd.png',             role: 'Mage' },
  { name: 'Mo & Krill',    icon: '/heroes/digger_sm_psd.png',             role: 'Brawler' },
  { name: 'Nurse Harrow',  icon: '/heroes/nurse_sm_psd.png',              role: 'Support',    update: 'City Never Sleeps' },
  { name: 'Paige',         icon: '/heroes/bookworm_sm_psd.png',           role: 'Support' },
  { name: 'Paradox',       icon: '/heroes/chrono_sm_psd.png',             role: 'Mage' },
  { name: 'Pocket',        icon: '/heroes/synth_sm_psd.png',              role: 'Mage' },
  { name: 'Rat King',      icon: '/heroes/ratking_sm_psd.png',            role: 'Brawler',    update: 'City Never Sleeps' },
  { name: 'Rem',           icon: '/heroes/familiar_sm_psd.png',           role: 'Support',    update: 'Old Gods New Blood' },
  { name: 'Seven',         icon: '/heroes/gigawatt_sm_psd.png',           role: 'Carry' },
  { name: 'Shiv',          icon: '/heroes/shiv_sm_psd.png',               role: 'Brawler' },
  { name: 'Silver',        icon: '/heroes/werewolf_sm_psd.png',           role: 'Carry',      update: 'Old Gods New Blood' },
  { name: 'Sinclair',      icon: '/heroes/magician_sm_psd.png',           role: 'Mage' },
  { name: 'Solomon',       icon: '/heroes/chessmaster_sm_psd.png',        role: 'Support',    update: 'City Never Sleeps' },
  { name: 'The Doorman',   icon: '/heroes/doorman_sm_psd.png',            role: 'Initiator' },
  { name: 'Venator',       icon: '/heroes/priest_sm_psd.png',             role: 'Initiator',  update: 'Old Gods New Blood' },
  { name: 'Victor',        icon: '/heroes/frank_sm_psd.png',              role: 'Brawler' },
  { name: 'Vindicta',      icon: '/heroes/hornet_sm_png.png',             role: 'Carry' },
  { name: 'Violet',        icon: '/heroes/artist_sm_psd.png',             role: 'Mage',       update: 'City Never Sleeps' },
  { name: 'Viscous',       icon: '/heroes/viscous_sm_psd.png',            role: 'Brawler' },
  { name: 'Vyper',         icon: '/heroes/kali_sm_psd.png',               role: 'Carry' },
  { name: 'Warden',        icon: '/heroes/warden_sm_psd.png',             role: 'Brawler' },
  { name: 'Wraith',        icon: '/heroes/wraith_sm_psd.png',             role: 'Carry' },
  { name: 'Wrecker',       icon: '',                                       role: 'Initiator' },
  { name: 'Yamato',        icon: '/heroes/yamato_sm_psd.png',             role: 'Initiator' },
];

/** Flat list of hero names (for dropdowns etc.) */
export const HERO_NAMES = DEADLOCK_HEROES.map((h) => h.name);

/** Map of name → icon path */
export const HERO_ICONS: Record<string, string> = Object.fromEntries(
  DEADLOCK_HEROES.map((h) => [h.name, h.icon])
);

/** Map of name → role */
export const HERO_ROLES: Record<string, string> = Object.fromEntries(
  DEADLOCK_HEROES.map((h) => [h.name, h.role])
);

export function getHeroIcon(name: string): string {
  return HERO_ICONS[name] ?? '';
}

/** Global/non-hero mod categories — no hero assignment required */
export const GLOBAL_CATEGORIES = ['ui', 'sounds', 'maps', 'hud', 'fonts', 'crosshairs', 'weapons'];

export function isGlobalCategory(category: string): boolean {
  return GLOBAL_CATEGORIES.includes(category.toLowerCase());
}
