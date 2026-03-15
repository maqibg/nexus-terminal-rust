import { tauriInvoke } from './invoke';

export const cryptoApi = {
  encrypt: (plaintext: string) => tauriInvoke<string>('crypto_encrypt', { plaintext }),
};

