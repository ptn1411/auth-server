import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { SecretDisplay } from '@/components/shared/SecretDisplay';

interface AppSecretDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appName: string;
  secret: string;
  isNewApp?: boolean;
}

export function AppSecretDialog({
  open,
  onOpenChange,
  appName,
  secret,
  isNewApp = false,
}: AppSecretDialogProps) {
  const title = isNewApp ? 'App Created Successfully' : 'New App Secret';
  const description = isNewApp
    ? `Your app "${appName}" has been created. Save the secret below.`
    : `A new secret has been generated for "${appName}".`;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <SecretDisplay
          title="App Secret"
          description={description}
          secret={secret}
          warning="This secret will only be shown once. Please save it securely. You will need it to authenticate your app with the API."
          onClose={() => onOpenChange(false)}
        />
      </DialogContent>
    </Dialog>
  );
}
