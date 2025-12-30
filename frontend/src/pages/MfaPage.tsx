import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { MfaForm } from '@/components/auth/MfaForm';

export function MfaPage() {
  return (
    <div className="flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">Two-Factor Authentication</CardTitle>
          <CardDescription>
            Enter the verification code from your authenticator app
          </CardDescription>
        </CardHeader>
        <CardContent>
          <MfaForm />
        </CardContent>
      </Card>
    </div>
  );
}
