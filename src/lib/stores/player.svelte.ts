// stores/player.svelte.ts — Steam / Statlocker player profile store (Svelte 5 Runes)
import { invoke } from '@tauri-apps/api/core';

export interface SteamAccountInfo {
  steam_id_64: string;
  account_id: number;
  account_name: string;
  persona_name: string;
  statlocker_url: string;
  is_detected: boolean;
}

let _player = $state<SteamAccountInfo | null>(null);
let _loading = $state(false);
let _error = $state('');

export function getPlayer(): SteamAccountInfo | null {
  return _player;
}

export function getPlayerLoading(): boolean {
  return _loading;
}

export function getPlayerError(): string {
  return _error;
}

export async function loadPlayer() {
  _loading = true;
  _error = '';
  try {
    const info = await invoke<SteamAccountInfo | null>('get_steam_account');
    const manualRaw =
      typeof localStorage !== 'undefined' ? localStorage.getItem('dmbk_manual_account_id') : null;
    const manualId = manualRaw ? Number(manualRaw) : 0;

    if (info) {
      _player =
        Number.isInteger(manualId) && manualId > 0
          ? {
              ...info,
              account_id: manualId,
              statlocker_url: `https://statlocker.net/players/${manualId}`,
            }
          : info;
    } else if (Number.isInteger(manualId) && manualId > 0) {
      _player = {
        steam_id_64: String(BigInt('76561197960265728') + BigInt(manualId)),
        account_id: manualId,
        account_name: `account_${manualId}`,
        persona_name: `Player #${manualId}`,
        statlocker_url: `https://statlocker.net/players/${manualId}`,
        is_detected: false,
      };
    } else {
      _player = null;
    }
  } catch (e) {
    _error = String(e);
    _player = null;
  } finally {
    _loading = false;
  }
}

export function setManualAccountId(accountId: number) {
  const parsed = Number(accountId);
  if (!Number.isInteger(parsed) || parsed <= 0) return;

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('dmbk_manual_account_id', String(parsed));
  }

  if (_player) {
    _player = {
      ..._player,
      account_id: parsed,
      statlocker_url: `https://statlocker.net/players/${parsed}`,
    };
  } else {
    _player = {
      steam_id_64: String(BigInt('76561197960265728') + BigInt(parsed)),
      account_id: parsed,
      account_name: `account_${parsed}`,
      persona_name: `Player #${parsed}`,
      statlocker_url: `https://statlocker.net/players/${parsed}`,
      is_detected: false,
    };
  }
}
