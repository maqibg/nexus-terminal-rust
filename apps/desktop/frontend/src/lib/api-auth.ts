/**
 * Auth API — replaces /api/v1/auth/* HTTP calls.
 */
import { tauriInvoke } from './invoke';

export interface AuthStatusResponse {
  is_authenticated: boolean;
  user?: { id: number; username: string; is_two_factor_enabled: boolean } | null;
}

export interface LoginResponse {
  requires_two_factor: boolean;
  user?: { id: number; username: string; is_two_factor_enabled: boolean } | null;
}

export interface TwoFactorSetupResponse {
  secret: string;
  url: string;
}

/** Derived auth state for the store */
export type AuthState = 'loading' | 'NeedsSetup' | 'Locked' | 'Authenticated' | 'error';

export interface PasskeyInfo {
  id: number;
  credential_id: string;
  name: string;
}

export const authApi = {
  /** Returns derived state + has_2fa from backend response */
  status: async (): Promise<{ state: AuthState; has_2fa: boolean }> => {
    try {
      const r = await tauriInvoke<AuthStatusResponse>('auth_status');
      if (r.is_authenticated && r.user) {
        return { state: 'Authenticated', has_2fa: r.user.is_two_factor_enabled };
      }
      return { state: 'Locked', has_2fa: false };
    } catch (e: any) {
      // Backend throws SetupRequired error when no user exists
      if (e.message?.includes('SetupRequired') || e.message?.includes('setup required')) {
        return { state: 'NeedsSetup', has_2fa: false };
      }
      throw e;
    }
  },

  setup: (username: string, password: string) =>
    tauriInvoke<unknown>('auth_setup', { req: { username, password } }),

  login: async (username: string, password: string): Promise<{ needs_2fa: boolean }> => {
    const r = await tauriInvoke<LoginResponse>('auth_login', { req: { username, password } });
    return { needs_2fa: r.requires_two_factor };
  },

  verify2fa: (token: string) =>
    tauriInvoke<unknown>('auth_verify_2fa', { req: { token } }),

  logout: () => tauriInvoke<void>('auth_logout'),

  changePassword: (currentPassword: string, newPassword: string) =>
    tauriInvoke<void>('auth_change_password', {
      req: { current_password: currentPassword, new_password: newPassword },
    }),

  setup2fa: () => tauriInvoke<TwoFactorSetupResponse>('auth_setup_2fa'),

  verifyActivate2fa: (token: string) =>
    tauriInvoke<void>('auth_verify_activate_2fa', { req: { token } }),

  disable2fa: (password: string) =>
    tauriInvoke<void>('auth_disable_2fa', { req: { password } }),

  // Passkey
  passkeyList: () => tauriInvoke<PasskeyInfo[]>('passkey_list'),
  passkeyRegisterStart: () => tauriInvoke<string>('passkey_register_start'),
  passkeyRegisterFinish: (credentialId: string, publicKey: string, name: string) =>
    tauriInvoke<number>('passkey_register_finish', { req: { credential_id: credentialId, public_key: publicKey, name } }),
  passkeyDelete: (credentialId: string) =>
    tauriInvoke<boolean>('passkey_delete', { req: { credential_id: credentialId } }),
  passkeyRename: (credentialId: string, name: string) =>
    tauriInvoke<boolean>('passkey_rename', { req: { credential_id: credentialId, name } }),
};
