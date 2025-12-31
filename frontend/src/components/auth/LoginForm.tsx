import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Link, useNavigate } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { useWebAuthn } from '@/hooks/useWebAuthn';
import { AuthServerError } from '@/lib/auth-client';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { toast } from 'sonner';

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(1, 'Password is required'),
});

type LoginFormValues = z.infer<typeof loginSchema>;

export function LoginForm() {
  const navigate = useNavigate();
  const { login, loginWithPasskey } = useAuthStore();
  const { isSupported: isPasskeySupported, authenticateWithPasskey, isLoading: isPasskeyLoading, error: passkeyError, clearError } = useWebAuthn();
  const [isLoading, setIsLoading] = useState(false);

  const form = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  });

  const onSubmit = async (data: LoginFormValues) => {
    setIsLoading(true);
    try {
      await login(data.email, data.password);
      
      // Check if MFA is required after login attempt
      const currentMfaPending = useAuthStore.getState().mfaPending;
      if (currentMfaPending) {
        navigate('/mfa');
      } else {
        toast.success('Login successful');
        navigate('/dashboard');
      }
    } catch (error) {
      if (error instanceof AuthServerError) {
        const message = getErrorMessage(error);
        toast.error(message);
      } else {
        toast.error('An unexpected error occurred');
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handlePasskeyLogin = async () => {
    clearError();
    const email = form.getValues('email') || undefined;
    
    const authResponse = await authenticateWithPasskey(email);
    
    if (authResponse) {
      try {
        await loginWithPasskey(authResponse.access_token, authResponse.refresh_token);
        toast.success('Login successful');
        navigate('/dashboard');
      } catch (error) {
        if (error instanceof AuthServerError) {
          toast.error(getErrorMessage(error));
        } else {
          toast.error('Failed to complete passkey login');
        }
      }
    } else if (passkeyError) {
      toast.error(passkeyError);
    }
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input
                  type="email"
                  placeholder="Enter your email"
                  autoComplete="email"
                  disabled={isLoading}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between">
                <FormLabel>Password</FormLabel>
                <Link
                  to="/forgot-password"
                  className="text-sm text-primary hover:underline"
                >
                  Forgot password?
                </Link>
              </div>
              <FormControl>
                <Input
                  type="password"
                  placeholder="Enter your password"
                  autoComplete="current-password"
                  disabled={isLoading}
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button type="submit" className="w-full" disabled={isLoading || isPasskeyLoading}>
          {isLoading ? (
            <span className="flex items-center gap-2">
              <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Signing in...
            </span>
          ) : (
            'Sign in'
          )}
        </Button>

        {isPasskeySupported && (
          <>
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">Or</span>
              </div>
            </div>

            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={handlePasskeyLogin}
              disabled={isLoading || isPasskeyLoading}
            >
              {isPasskeyLoading ? (
                <span className="flex items-center gap-2">
                  <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Authenticating...
                </span>
              ) : (
                <span className="flex items-center gap-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M2 18v3c0 .6.4 1 1 1h4v-3h3v-3h2l1.4-1.4a6.5 6.5 0 1 0-4-4Z" />
                    <circle cx="16.5" cy="7.5" r=".5" fill="currentColor" />
                  </svg>
                  Sign in with Passkey
                </span>
              )}
            </Button>
            
            <p className="text-xs text-center text-muted-foreground">
              Click above to use fingerprint, Face ID, or scan QR with your phone
            </p>
          </>
        )}

        <p className="text-center text-sm text-muted-foreground">
          Don't have an account?{' '}
          <Link to="/register" className="text-primary hover:underline">
            Sign up
          </Link>
        </p>
      </form>
    </Form>
  );
}

function getErrorMessage(error: AuthServerError): string {
  switch (error.error) {
    case 'invalid_credentials':
      return 'Invalid email or password';
    case 'account_locked':
      return 'Account is locked. Please try again later';
    case 'email_not_verified':
      return 'Please verify your email before logging in';
    case 'user_inactive':
      return 'Your account has been deactivated';
    case 'rate_limit':
      return 'Too many attempts. Please try again later';
    default:
      return error.message || 'Login failed. Please try again';
  }
}
