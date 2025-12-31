import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { DialogFooter } from '@/components/ui/dialog';
import { useWebAuthn } from '@/hooks/useWebAuthn';
import type { PasskeyResponse } from '@/lib/auth-client';
import { toast } from 'sonner';
import { Loader2, Key, Smartphone } from 'lucide-react';

interface PasskeyRegisterProps {
  onSuccess: (passkey: PasskeyResponse) => void;
  onCancel: () => void;
}

export function PasskeyRegister({ onSuccess, onCancel }: PasskeyRegisterProps) {
  const { registerPasskey, isLoading, error } = useWebAuthn();
  const [deviceName, setDeviceName] = useState('');
  const [isRegistering, setIsRegistering] = useState(false);

  const handleRegister = async () => {
    setIsRegistering(true);
    const passkey = await registerPasskey(deviceName.trim() || undefined);
    
    if (passkey) {
      toast.success('Passkey registered successfully');
      onSuccess(passkey);
    } else {
      toast.error(error || 'Failed to register passkey');
    }
    setIsRegistering(false);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-center py-4">
        <div className="p-4 bg-muted rounded-full">
          <Key className="h-8 w-8 text-primary" />
        </div>
      </div>

      <p className="text-sm text-muted-foreground text-center">
        Register a passkey for passwordless login. You can use this device or scan a QR code with your phone.
      </p>

      <div className="space-y-2">
        <Label htmlFor="device-name">Device Name (optional)</Label>
        <Input
          id="device-name"
          value={deviceName}
          onChange={(e) => setDeviceName(e.target.value)}
          placeholder="e.g., MacBook Pro, iPhone"
        />
        <p className="text-xs text-muted-foreground">
          Give your passkey a name to help identify it later.
        </p>
      </div>

      {error && (
        <p className="text-sm text-destructive text-center">{error}</p>
      )}

      <div className="bg-muted/50 rounded-lg p-3 space-y-2">
        <div className="flex items-center gap-2 text-sm">
          <Smartphone className="h-4 w-4 text-muted-foreground" />
          <span className="font-medium">Want to use your iPhone or Android?</span>
        </div>
        <p className="text-xs text-muted-foreground">
          Click "Register Passkey" below. Your browser will show a QR code that you can scan with your phone's camera to register a passkey from that device.
        </p>
      </div>

      <DialogFooter>
        <Button
          variant="outline"
          onClick={onCancel}
          disabled={isRegistering || isLoading}
        >
          Cancel
        </Button>
        <Button
          onClick={handleRegister}
          disabled={isRegistering || isLoading}
        >
          {isRegistering || isLoading ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              Registering...
            </>
          ) : (
            'Register Passkey'
          )}
        </Button>
      </DialogFooter>
    </div>
  );
}
