import { useEffect, useState, useMemo } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { authClient, AuthServerError } from '@/lib/auth-client';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

type VerificationStatus = 'loading' | 'success' | 'error' | 'no-token';

export function VerifyEmailPage() {
  const [searchParams] = useSearchParams();
  const token = searchParams.get('token');
  
  // Compute initial status based on token presence
  const initialStatus = useMemo<VerificationStatus>(() => {
    return token ? 'loading' : 'no-token';
  }, [token]);
  
  const [status, setStatus] = useState<VerificationStatus>(initialStatus);
  const [errorMessage, setErrorMessage] = useState('');

  useEffect(() => {
    if (!token) {
      return;
    }

    const verifyEmail = async () => {
      try {
        await authClient.verifyEmail({ token });
        setStatus('success');
      } catch (error) {
        setStatus('error');
        if (error instanceof AuthServerError) {
          setErrorMessage(getErrorMessage(error));
        } else {
          setErrorMessage('An unexpected error occurred');
        }
      }
    };

    verifyEmail();
  }, [token]);

  return (
    <div className="flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">Email Verification</CardTitle>
          <CardDescription>
            {status === 'loading' && 'Verifying your email address...'}
            {status === 'success' && 'Your email has been verified'}
            {status === 'error' && 'Verification failed'}
            {status === 'no-token' && 'Invalid verification link'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {status === 'loading' && (
            <div className="flex flex-col items-center gap-4">
              <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
              <p className="text-sm text-muted-foreground">
                Please wait while we verify your email...
              </p>
            </div>
          )}

          {status === 'success' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-green-100 dark:bg-green-900">
                <svg
                  className="h-6 w-6 text-green-600 dark:text-green-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <p className="text-sm text-muted-foreground">
                Your email has been successfully verified. You can now sign in to your account.
              </p>
              <Button asChild className="w-full">
                <Link to="/login">Sign in</Link>
              </Button>
            </div>
          )}

          {status === 'error' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-red-100 dark:bg-red-900">
                <svg
                  className="h-6 w-6 text-red-600 dark:text-red-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </div>
              <p className="text-sm text-muted-foreground">{errorMessage}</p>
              <div className="flex flex-col gap-2">
                <Button asChild variant="outline" className="w-full">
                  <Link to="/login">Go to login</Link>
                </Button>
              </div>
            </div>
          )}

          {status === 'no-token' && (
            <div className="space-y-4 text-center">
              <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-yellow-100 dark:bg-yellow-900">
                <svg
                  className="h-6 w-6 text-yellow-600 dark:text-yellow-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
              </div>
              <p className="text-sm text-muted-foreground">
                The verification link is invalid or missing. Please check your email for the correct link.
              </p>
              <Button asChild variant="outline" className="w-full">
                <Link to="/login">Go to login</Link>
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function getErrorMessage(error: AuthServerError): string {
  switch (error.error) {
    case 'invalid_token':
    case 'token_expired':
      return 'This verification link has expired or is invalid. Please request a new verification email.';
    case 'already_verified':
      return 'This email has already been verified. You can sign in to your account.';
    default:
      return error.message || 'Email verification failed. Please try again.';
  }
}
