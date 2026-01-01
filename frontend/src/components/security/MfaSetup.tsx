import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { authClient, type TotpSetupResponse } from '@/lib/auth-client';
import { useAuthStore } from '@/stores/authStore';
import { toast } from 'sonner';
import { Shield, Loader2, Copy, Check, Key } from 'lucide-react';
import { BackupCodes } from './BackupCodes';
import QRCode from 'qrcode';

export function MfaSetup() {
  const { user, refreshUser } = useAuthStore();
  const [isLoading, setIsLoading] = useState(false);
  const [setupData, setSetupData] = useState<TotpSetupResponse | null>(null);
  const [qrCodeDataUrl, setQrCodeDataUrl] = useState<string | null>(null);
  const [verificationCode, setVerificationCode] = useState('');
  const [backupCodes, setBackupCodes] = useState<string[] | null>(null);
  const [showSetupDialog, setShowSetupDialog] = useState(false);
  const [showDisableDialog, setShowDisableDialog] = useState(false);
  const [showBackupCodesDialog, setShowBackupCodesDialog] = useState(false);
  const [secretCopied, setSecretCopied] = useState(false);
  const [isVerifying, setIsVerifying] = useState(false);
  const [isDisabling, setIsDisabling] = useState(false);
  const [isRegenerating, setIsRegenerating] = useState(false);

  // Generate QR code when setupData changes
  useEffect(() => {
    if (setupData?.provisioning_uri) {
      QRCode.toDataURL(setupData.provisioning_uri, {
        width: 200,
        margin: 1,
      }).then(setQrCodeDataUrl).catch(console.error);
    } else {
      setQrCodeDataUrl(null);
    }
  }, [setupData?.provisioning_uri]);

  const handleStartSetup = async () => {
    setIsLoading(true);
    try {
      const response = await authClient.setupTotp();
      setSetupData(response);
      setShowSetupDialog(true);
    } catch (error) {
      toast.error('Failed to start MFA setup');
      console.error('Failed to start MFA setup:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerifySetup = async () => {
    if (!setupData || verificationCode.length !== 6) return;

    setIsVerifying(true);
    try {
      const response = await authClient.verifyTotpSetup({
        method_id: setupData.method_id,
        code: verificationCode,
      });
      setBackupCodes(response.backup_codes);
      setShowSetupDialog(false);
      setShowBackupCodesDialog(true);
      await refreshUser();
      toast.success('MFA enabled successfully');
    } catch (error) {
      toast.error('Invalid verification code');
      console.error('Failed to verify TOTP:', error);
    } finally {
      setIsVerifying(false);
    }
  };

  const handleDisableMfa = async () => {
    setIsDisabling(true);
    try {
      await authClient.disableMfa();
      await refreshUser();
      setShowDisableDialog(false);
      toast.success('MFA disabled successfully');
    } catch (error) {
      toast.error('Failed to disable MFA');
      console.error('Failed to disable MFA:', error);
    } finally {
      setIsDisabling(false);
    }
  };

  const handleRegenerateBackupCodes = async () => {
    setIsRegenerating(true);
    try {
      const response = await authClient.regenerateBackupCodes();
      setBackupCodes(response.backup_codes);
      setShowBackupCodesDialog(true);
      toast.success('Backup codes regenerated');
    } catch (error) {
      toast.error('Failed to regenerate backup codes');
      console.error('Failed to regenerate backup codes:', error);
    } finally {
      setIsRegenerating(false);
    }
  };

  const copySecret = async () => {
    if (!setupData) return;
    try {
      await navigator.clipboard.writeText(setupData.secret);
      setSecretCopied(true);
      setTimeout(() => setSecretCopied(false), 2000);
      toast.success('Secret copied to clipboard');
    } catch {
      toast.error('Failed to copy secret');
    }
  };

  const handleCloseSetupDialog = () => {
    setShowSetupDialog(false);
    setSetupData(null);
    setVerificationCode('');
  };

  const handleCloseBackupCodesDialog = () => {
    setShowBackupCodesDialog(false);
    setBackupCodes(null);
  };

  return (
    <>
      <Card>
        <CardHeader className="pb-3 sm:pb-6">
          <CardTitle className="flex items-center gap-2 text-base sm:text-lg">
            <Shield className="h-4 w-4 sm:h-5 sm:w-5" />
            Two-Factor Authentication
          </CardTitle>
          <CardDescription className="text-xs sm:text-sm">
            Add an extra layer of security to your account
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div className="space-y-1">
              <p className="font-medium text-sm sm:text-base">TOTP Authenticator</p>
              <p className="text-xs sm:text-sm text-muted-foreground">
                Use an authenticator app like Google Authenticator or Authy
              </p>
            </div>
            <Badge variant={user?.mfa_enabled ? 'default' : 'secondary'} className="w-fit">
              {user?.mfa_enabled ? 'Enabled' : 'Disabled'}
            </Badge>
          </div>

          <div className="flex flex-col sm:flex-row gap-2">
            {user?.mfa_enabled ? (
              <>
                <Button
                  variant="outline"
                  onClick={handleRegenerateBackupCodes}
                  disabled={isRegenerating}
                  className="w-full sm:w-auto"
                >
                  {isRegenerating ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Regenerating...
                    </>
                  ) : (
                    <>
                      <Key className="h-4 w-4 mr-2" />
                      Regenerate Backup Codes
                    </>
                  )}
                </Button>
                <Button
                  variant="destructive"
                  onClick={() => setShowDisableDialog(true)}
                  className="w-full sm:w-auto"
                >
                  Disable MFA
                </Button>
              </>
            ) : (
              <Button onClick={handleStartSetup} disabled={isLoading} className="w-full sm:w-auto">
                {isLoading ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Setting up...
                  </>
                ) : (
                  <>
                    <Shield className="h-4 w-4 mr-2" />
                    Enable MFA
                  </>
                )}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Setup Dialog */}
      <Dialog open={showSetupDialog} onOpenChange={handleCloseSetupDialog}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Set Up Two-Factor Authentication</DialogTitle>
            <DialogDescription>
              Scan the QR code with your authenticator app or enter the secret manually.
            </DialogDescription>
          </DialogHeader>
          
          {setupData && (
            <div className="space-y-4">
              <div className="flex justify-center">
                <div className="p-4 bg-white rounded-lg">
                  {qrCodeDataUrl ? (
                    <img
                      src={qrCodeDataUrl}
                      alt="QR Code"
                      className="w-40 h-40 sm:w-48 sm:h-48"
                    />
                  ) : (
                    <div className="w-40 h-40 sm:w-48 sm:h-48 flex items-center justify-center">
                      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                    </div>
                  )}
                </div>
              </div>

              <div className="space-y-2">
                <Label>Manual Entry Secret</Label>
                <div className="flex gap-2">
                  <Input
                    value={setupData.secret}
                    readOnly
                    className="font-mono text-xs sm:text-sm"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    onClick={copySecret}
                  >
                    {secretCopied ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="verification-code">Verification Code</Label>
                <Input
                  id="verification-code"
                  placeholder="Enter 6-digit code"
                  value={verificationCode}
                  onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  maxLength={6}
                  className="text-center text-lg tracking-widest"
                />
              </div>
            </div>
          )}

          <DialogFooter className="flex-col sm:flex-row gap-2">
            <Button
              variant="outline"
              onClick={handleCloseSetupDialog}
              disabled={isVerifying}
              className="w-full sm:w-auto"
            >
              Cancel
            </Button>
            <Button
              onClick={handleVerifySetup}
              disabled={verificationCode.length !== 6 || isVerifying}
              className="w-full sm:w-auto"
            >
              {isVerifying ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Verifying...
                </>
              ) : (
                'Verify & Enable'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Disable MFA Dialog */}
      <Dialog open={showDisableDialog} onOpenChange={setShowDisableDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Disable Two-Factor Authentication</DialogTitle>
            <DialogDescription>
              Are you sure you want to disable MFA? This will make your account less secure.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="flex-col sm:flex-row gap-2">
            <Button
              variant="outline"
              onClick={() => setShowDisableDialog(false)}
              disabled={isDisabling}
              className="w-full sm:w-auto"
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDisableMfa}
              disabled={isDisabling}
              className="w-full sm:w-auto"
            >
              {isDisabling ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Disabling...
                </>
              ) : (
                'Disable MFA'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Backup Codes Dialog */}
      <Dialog open={showBackupCodesDialog} onOpenChange={handleCloseBackupCodesDialog}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Backup Codes</DialogTitle>
            <DialogDescription>
              Save these backup codes in a secure place. You can use them to access your account if you lose your authenticator device.
            </DialogDescription>
          </DialogHeader>
          
          {backupCodes && <BackupCodes codes={backupCodes} />}

          <DialogFooter>
            <Button onClick={handleCloseBackupCodesDialog} className="w-full sm:w-auto">
              I've Saved My Codes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
