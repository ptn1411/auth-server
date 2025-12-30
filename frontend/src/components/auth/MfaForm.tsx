import { useState, useRef, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { AuthServerError } from '@/lib/auth-client';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { toast } from 'sonner';
import { cn } from '@/lib/utils';

interface MfaFormProps {
  onCancel?: () => void;
}

export function MfaForm({ onCancel }: MfaFormProps) {
  const navigate = useNavigate();
  const { completeMfa, mfaPending, clearMfaPending } = useAuthStore();
  const [code, setCode] = useState(['', '', '', '', '', '']);
  const [isLoading, setIsLoading] = useState(false);
  const [useBackupCode, setUseBackupCode] = useState(false);
  const [backupCode, setBackupCode] = useState('');
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  // Redirect if no MFA pending
  useEffect(() => {
    if (!mfaPending) {
      navigate('/login');
    }
  }, [mfaPending, navigate]);

  const submitCode = useCallback(async (codeString: string) => {
    if (!codeString) return;

    setIsLoading(true);
    try {
      await completeMfa(codeString);
      toast.success('Verification successful');
      navigate('/dashboard');
    } catch (error) {
      if (error instanceof AuthServerError) {
        const message = getErrorMessage(error);
        toast.error(message);
        // Clear code on error
        setCode(['', '', '', '', '', '']);
        inputRefs.current[0]?.focus();
      } else {
        toast.error('An unexpected error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  }, [completeMfa, navigate]);

  // Auto-submit when all 6 digits are entered
  useEffect(() => {
    if (!useBackupCode && code.every(digit => digit !== '')) {
      const codeString = code.join('');
      if (codeString.length === 6) {
        submitCode(codeString);
      }
    }
  }, [code, useBackupCode, submitCode]);

  const handleDigitChange = (index: number, value: string) => {
    // Only allow single digit
    const digit = value.slice(-1);
    if (digit && !/^\d$/.test(digit)) return;

    const newCode = [...code];
    newCode[index] = digit;
    setCode(newCode);

    // Move to next input
    if (digit && index < 5) {
      inputRefs.current[index + 1]?.focus();
    }
  };

  const handleKeyDown = (index: number, e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Backspace' && !code[index] && index > 0) {
      inputRefs.current[index - 1]?.focus();
    }
  };

  const handlePaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const pastedData = e.clipboardData.getData('text').replace(/\D/g, '').slice(0, 6);
    if (pastedData.length === 6) {
      setCode(pastedData.split(''));
    }
  };

  const handleSubmit = async () => {
    const codeString = useBackupCode ? backupCode.trim() : code.join('');
    
    if (!codeString || (useBackupCode && codeString.length < 8) || (!useBackupCode && codeString.length !== 6)) {
      return;
    }

    await submitCode(codeString);
  };

  const handleCancel = () => {
    clearMfaPending();
    if (onCancel) {
      onCancel();
    } else {
      navigate('/login');
    }
  };

  if (!mfaPending) {
    return null;
  }

  return (
    <div className="space-y-6">
      {!useBackupCode ? (
        <div className="space-y-4">
          <div className="flex justify-center gap-2" onPaste={handlePaste}>
            {code.map((digit, index) => (
              <Input
                key={index}
                ref={(el) => { inputRefs.current[index] = el; }}
                type="text"
                inputMode="numeric"
                maxLength={1}
                value={digit}
                onChange={(e) => handleDigitChange(index, e.target.value)}
                onKeyDown={(e) => handleKeyDown(index, e)}
                disabled={isLoading}
                className={cn(
                  'h-12 w-12 text-center text-lg font-semibold',
                  'focus:ring-2 focus:ring-primary'
                )}
                autoFocus={index === 0}
              />
            ))}
          </div>

          <Button
            type="button"
            className="w-full"
            onClick={handleSubmit}
            disabled={isLoading || code.some(d => !d)}
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                Verifying...
              </span>
            ) : (
              'Verify'
            )}
          </Button>
        </div>
      ) : (
        <div className="space-y-4">
          <div>
            <Input
              type="text"
              placeholder="Enter backup code"
              value={backupCode}
              onChange={(e) => setBackupCode(e.target.value)}
              disabled={isLoading}
              autoFocus
            />
          </div>

          <Button
            type="button"
            className="w-full"
            onClick={handleSubmit}
            disabled={isLoading || !backupCode.trim()}
          >
            {isLoading ? (
              <span className="flex items-center gap-2">
                <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                Verifying...
              </span>
            ) : (
              'Verify with backup code'
            )}
          </Button>
        </div>
      )}

      <div className="flex flex-col gap-2">
        <Button
          type="button"
          variant="ghost"
          className="w-full"
          onClick={() => {
            setUseBackupCode(!useBackupCode);
            setCode(['', '', '', '', '', '']);
            setBackupCode('');
          }}
          disabled={isLoading}
        >
          {useBackupCode ? 'Use authenticator app' : 'Use backup code instead'}
        </Button>

        <Button
          type="button"
          variant="outline"
          className="w-full"
          onClick={handleCancel}
          disabled={isLoading}
        >
          Cancel
        </Button>
      </div>
    </div>
  );
}

function getErrorMessage(error: AuthServerError): string {
  switch (error.error) {
    case 'invalid_mfa_code':
      return 'Invalid verification code';
    case 'mfa_token_expired':
      return 'Session expired. Please login again';
    case 'rate_limit':
      return 'Too many attempts. Please try again later';
    default:
      return error.message || 'Verification failed. Please try again';
  }
}
