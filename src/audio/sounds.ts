/**
 * Sound effect identifiers for the game
 */
export enum SoundEffect {
  // UI sounds
  ButtonClick = 'button_click',
  ButtonHover = 'button_hover',
  MenuOpen = 'menu_open',
  MenuClose = 'menu_close',

  // Game sounds
  UnitSelect = 'unit_select',
  UnitMove = 'unit_move',
  UnitAttack = 'unit_attack',
  UnitDeath = 'unit_death',
  CityFounded = 'city_founded',
  BuildingComplete = 'building_complete',
  TechResearched = 'tech_researched',
  TurnStart = 'turn_start',
  Victory = 'victory',
  Defeat = 'defeat',

  // Notifications
  NotificationInfo = 'notification_info',
  NotificationWarning = 'notification_warning',
  NotificationError = 'notification_error',
}

/**
 * Music track identifiers for the game
 */
export enum MusicTrack {
  MainMenu = 'main_menu',
  GameAmbient = 'game_ambient',
  Combat = 'combat',
  Victory = 'victory',
  Defeat = 'defeat',
}

/**
 * Sound file paths (placeholders until actual audio files are added)
 * These paths are relative to the public/audio directory
 */
export const SOUND_PATHS: Record<SoundEffect, string> = {
  // UI sounds
  [SoundEffect.ButtonClick]: '/audio/sfx/ui/button_click.mp3',
  [SoundEffect.ButtonHover]: '/audio/sfx/ui/button_hover.mp3',
  [SoundEffect.MenuOpen]: '/audio/sfx/ui/menu_open.mp3',
  [SoundEffect.MenuClose]: '/audio/sfx/ui/menu_close.mp3',

  // Game sounds
  [SoundEffect.UnitSelect]: '/audio/sfx/game/unit_select.mp3',
  [SoundEffect.UnitMove]: '/audio/sfx/game/unit_move.mp3',
  [SoundEffect.UnitAttack]: '/audio/sfx/game/unit_attack.mp3',
  [SoundEffect.UnitDeath]: '/audio/sfx/game/unit_death.mp3',
  [SoundEffect.CityFounded]: '/audio/sfx/game/city_founded.mp3',
  [SoundEffect.BuildingComplete]: '/audio/sfx/game/building_complete.mp3',
  [SoundEffect.TechResearched]: '/audio/sfx/game/tech_researched.mp3',
  [SoundEffect.TurnStart]: '/audio/sfx/game/turn_start.mp3',
  [SoundEffect.Victory]: '/audio/sfx/game/victory.mp3',
  [SoundEffect.Defeat]: '/audio/sfx/game/defeat.mp3',

  // Notifications
  [SoundEffect.NotificationInfo]: '/audio/sfx/notifications/info.mp3',
  [SoundEffect.NotificationWarning]: '/audio/sfx/notifications/warning.mp3',
  [SoundEffect.NotificationError]: '/audio/sfx/notifications/error.mp3',
}

/**
 * Music file paths (placeholders until actual audio files are added)
 * These paths are relative to the public/audio directory
 */
export const MUSIC_PATHS: Record<MusicTrack, string> = {
  [MusicTrack.MainMenu]: '/audio/music/main_menu.mp3',
  [MusicTrack.GameAmbient]: '/audio/music/game_ambient.mp3',
  [MusicTrack.Combat]: '/audio/music/combat.mp3',
  [MusicTrack.Victory]: '/audio/music/victory.mp3',
  [MusicTrack.Defeat]: '/audio/music/defeat.mp3',
}
