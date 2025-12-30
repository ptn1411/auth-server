import { useState, useCallback } from 'react';
import { authClient } from '@/lib/auth-client';
import type {
  RegistrationOptionsResponse,
  AuthenticationOptionsResponse,
  PasskeyResponse,
  PasskeyAuthResponse,
} from '@/lib/auth-client';

// Helper to convert base64url to ArrayBuffer
function base64urlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
  const padding = '='.repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(base64 + padding);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

// Helper to convert ArrayBuffer to base64url
function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary);
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

interface UseWebAuthnReturn {
  isSupported: boolean;
  isLoading: boolean;
  error: string | null;
  registerPasskey: (deviceName?: string) => Promise<PasskeyResponse | null>;
  authenticateWithPasskey: (email?: string) => Promise<PasskeyAuthResponse | null>;
  listPasskeys: () => Promise<PasskeyResponse[]>;
  deletePasskey: (credentialId: string) => Promise<boolean>;
  renamePasskey: (credentialId: string, name: string) => Promise<boolean>;
  clearError: () => void;
}

export function useWebAuthn(): UseWebAuthnReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check if WebAuthn is supported
  const isSupported = typeof window !== 'undefined' && 
    !!window.PublicKeyCredential &&
    typeof window.PublicKeyCredential === 'function';

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const registerPasskey = useCallback(async (deviceName?: string): Promise<PasskeyResponse | null> => {
    if (!isSupported) {
      setError('WebAuthn is not supported in this browser');
      return null;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Step 1: Get registration options from server
      const options: RegistrationOptionsResponse = await authClient.startPasskeyRegistration({
        device_name: deviceName,
      });

      // Step 2: Create credential using WebAuthn API
      const publicKeyCredentialCreationOptions: PublicKeyCredentialCreationOptions = {
        challenge: base64urlToBuffer(options.challenge),
        rp: {
          id: options.rp.id,
          name: options.rp.name,
        },
        user: {
          id: base64urlToBuffer(options.user.id),
          name: options.user.name,
          displayName: options.user.display_name,
        },
        pubKeyCredParams: options.pub_key_cred_params.map(param => ({
          type: param.type as PublicKeyCredentialType,
          alg: param.alg,
        })),
        timeout: options.timeout,
        attestation: options.attestation as AttestationConveyancePreference,
        authenticatorSelection: {
          authenticatorAttachment: options.authenticator_selection.authenticator_attachment as AuthenticatorAttachment | undefined,
          residentKey: options.authenticator_selection.resident_key as ResidentKeyRequirement,
          userVerification: options.authenticator_selection.user_verification as UserVerificationRequirement,
        },
      };

      const credential = await navigator.credentials.create({
        publicKey: publicKeyCredentialCreationOptions,
      }) as PublicKeyCredential | null;

      if (!credential) {
        throw new Error('Failed to create credential');
      }

      const response = credential.response as AuthenticatorAttestationResponse;

      // Step 3: Send credential to server for verification
      const passkey = await authClient.finishPasskeyRegistration({
        id: credential.id,
        raw_id: bufferToBase64url(credential.rawId),
        response: {
          client_data_json: bufferToBase64url(response.clientDataJSON),
          attestation_object: bufferToBase64url(response.attestationObject),
        },
        type: credential.type,
        device_name: deviceName,
      });

      return passkey;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to register passkey';
      setError(message);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [isSupported]);

  const authenticateWithPasskey = useCallback(async (email?: string): Promise<PasskeyAuthResponse | null> => {
    if (!isSupported) {
      setError('WebAuthn is not supported in this browser');
      return null;
    }

    setIsLoading(true);
    setError(null);

    try {
      // Step 1: Get authentication options from server
      const options: AuthenticationOptionsResponse = await authClient.startPasskeyAuthentication({
        email,
      });

      // Step 2: Get credential using WebAuthn API
      const publicKeyCredentialRequestOptions: PublicKeyCredentialRequestOptions = {
        challenge: base64urlToBuffer(options.challenge),
        timeout: options.timeout,
        rpId: options.rp_id,
        allowCredentials: options.allow_credentials.map(cred => ({
          id: base64urlToBuffer(cred.id),
          type: cred.type as PublicKeyCredentialType,
          transports: cred.transports as AuthenticatorTransport[] | undefined,
        })),
        userVerification: options.user_verification as UserVerificationRequirement,
      };

      const credential = await navigator.credentials.get({
        publicKey: publicKeyCredentialRequestOptions,
      }) as PublicKeyCredential | null;

      if (!credential) {
        throw new Error('Failed to get credential');
      }

      const response = credential.response as AuthenticatorAssertionResponse;

      // Step 3: Send credential to server for verification
      const authResponse = await authClient.finishPasskeyAuthentication({
        id: credential.id,
        raw_id: bufferToBase64url(credential.rawId),
        response: {
          client_data_json: bufferToBase64url(response.clientDataJSON),
          authenticator_data: bufferToBase64url(response.authenticatorData),
          signature: bufferToBase64url(response.signature),
          user_handle: response.userHandle ? bufferToBase64url(response.userHandle) : undefined,
        },
        type: credential.type,
      });

      return authResponse;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to authenticate with passkey';
      setError(message);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [isSupported]);

  const listPasskeys = useCallback(async (): Promise<PasskeyResponse[]> => {
    setIsLoading(true);
    setError(null);

    try {
      const passkeys = await authClient.listPasskeys();
      return passkeys;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to list passkeys';
      setError(message);
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  const deletePasskey = useCallback(async (credentialId: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);

    try {
      await authClient.deletePasskey(credentialId);
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete passkey';
      setError(message);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const renamePasskey = useCallback(async (credentialId: string, name: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);

    try {
      await authClient.renamePasskey(credentialId, { name });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to rename passkey';
      setError(message);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    isSupported,
    isLoading,
    error,
    registerPasskey,
    authenticateWithPasskey,
    listPasskeys,
    deletePasskey,
    renamePasskey,
    clearError,
  };
}
