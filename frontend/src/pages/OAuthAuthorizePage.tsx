import { useEffect, useState } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAuthStore } from '@/stores/authStore';
import { useOAuthClientsStore, type ConsentRequiredResponse } from '@/stores/oauthClientsStore';
import { toast } from 'sonner';
import { Shield, CheckCircle2, XCircle, Loader2, AlertTriangle, ExternalLink } from 'lucide-react';

// Scope descriptions for display
const SCOPE_DESCRIPTIONS: Record<string, { name: string; description: string }> = {
  'openid': { name: 'OpenID', description: 'Verify your identity' },
  'profile': { name: 'Profile', description: 'Access your basic profile information (name)' },
  'email': { name: 'Email', description: 'Access your email address' },
  'profile.read': { name: 'Read Profile', description: 'Read your profile information' },
  'email.read': { name: 'Read Email', description: 'Read your email address' },
};

export function OAuthAuthorizePage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { user, isAuthenticated } = useAuthStore();
  const { initiateAuthorization, submitConsent, isLoading, error } = useOAuthClientsStore();
  
  const [consentData, setConsentData] = useState<ConsentRequiredResponse | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);

  // Get params from URL
  const clientId = searchParams.get('client_id') || '';
  const responseType = searchParams.get('response_type') || '';
  const redirectUri = searchParams.get('redirect_uri') || '';
  const scope = searchParams.get('scope') || '';
  const state = searchParams.get('state') || undefined;
  const codeChallenge = searchParams.get('code_challenge') || undefined;
  const codeChallengeMethod = searchParams.get('code_challenge_method') || undefined;

  useEffect(() => {
    // If not authenticated, redirect to login with return URL
    if (!isAuthenticated) {
      const returnUrl = window.location.pathname + window.location.search;
      navigate(`/login?returnUrl=${encodeURIComponent(returnUrl)}`);
      return;
    }

    // Validate required params
    if (!clientId || !responseType || !redirectUri || !scope) {
      setAuthError('Missing required parameters');
      return;
    }

    if (responseType !== 'code') {
      setAuthError('Unsupported response type. Only "code" is supported.');
      return;
    }

    // Initiate authorization to get consent info
    const initAuth = async () => {
      try {
        const data = await initiateAuthorization({
          client_id: clientId,
          response_type: responseType,
          redirect_uri: redirectUri,
          scope,
          state,
          code_challenge: codeChallenge,
          code_challenge_method: codeChallengeMethod,
        });
        setConsentData(data);
      } catch (err) {
        setAuthError(err instanceof Error ? err.message : 'Failed to initiate authorization');
      }
    };

    initAuth();
  }, [isAuthenticated, clientId, responseType, redirectUri, scope, state, codeChallenge, codeChallengeMethod, navigate, initiateAuthorization]);

  const handleConsent = async (approved: boolean) => {
    if (!consentData || !user) return;

    setIsSubmitting(true);
    try {
      const redirectUrl = await submitConsent({
        approved,
        client_id: consentData.client_id,
        user_id: user.id,
        redirect_uri: consentData.redirect_uri,
        scopes: consentData.scopes.join(','),
        state: consentData.state,
        code_challenge: consentData.code_challenge,
        code_challenge_method: consentData.code_challenge_method,
      });

      if (redirectUrl) {
        // Redirect to the app's callback URL
        window.location.href = redirectUrl;
      } else if (!approved) {
        // If denied and no redirect, show message
        toast.info('Authorization denied');
        navigate('/dashboard');
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to process consent');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Error state
  if (authError || error) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 h-12 w-12 rounded-full bg-destructive/10 flex items-center justify-center">
              <AlertTriangle className="h-6 w-6 text-destructive" />
            </div>
            <CardTitle>Authorization Error</CardTitle>
            <CardDescription>
              {authError || error}
            </CardDescription>
          </CardHeader>
          <CardFooter className="justify-center">
            <Button variant="outline" onClick={() => navigate('/dashboard')}>
              Return to Dashboard
            </Button>
          </CardFooter>
        </Card>
      </div>
    );
  }

  // Loading state
  if (isLoading || !consentData) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-primary" />
          <p className="text-muted-foreground">Loading authorization request...</p>
        </div>
      </div>
    );
  }

  const scopes = consentData.scopes;

  return (
    <div className="min-h-screen flex items-center justify-center p-4 bg-muted/30">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center">
            <Shield className="h-8 w-8 text-primary" />
          </div>
          <CardTitle className="text-xl">Authorize Application</CardTitle>
          <CardDescription>
            <span className="font-semibold text-foreground">{consentData.client_name}</span>
            {' '}is requesting access to your account
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* User info */}
          <div className="p-3 rounded-lg bg-muted/50 text-sm">
            <span className="text-muted-foreground">Signed in as: </span>
            <span className="font-medium">{user?.email}</span>
          </div>

          {/* Requested permissions */}
          <div>
            <h4 className="text-sm font-medium mb-3">This application will be able to:</h4>
            <div className="space-y-2">
              {scopes.map((scopeCode) => {
                const scopeInfo = SCOPE_DESCRIPTIONS[scopeCode] || {
                  name: scopeCode,
                  description: `Access ${scopeCode}`,
                };
                return (
                  <div
                    key={scopeCode}
                    className="flex items-start gap-3 p-3 rounded-lg bg-muted/50"
                  >
                    <CheckCircle2 className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                    <div>
                      <p className="font-medium text-sm">{scopeInfo.name}</p>
                      <p className="text-xs text-muted-foreground">{scopeInfo.description}</p>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          <div className="border-t my-4" />

          {/* Redirect info */}
          <div className="text-xs text-muted-foreground">
            <p className="flex items-center gap-1">
              <ExternalLink className="h-3 w-3" />
              After authorization, you'll be redirected to:
            </p>
            <Badge variant="secondary" className="mt-1 font-mono text-xs break-all">
              {consentData.redirect_uri}
            </Badge>
          </div>
        </CardContent>

        <CardFooter className="flex gap-3">
          <Button
            variant="outline"
            className="flex-1"
            onClick={() => handleConsent(false)}
            disabled={isSubmitting}
          >
            <XCircle className="h-4 w-4 mr-2" />
            Deny
          </Button>
          <Button
            className="flex-1"
            onClick={() => handleConsent(true)}
            disabled={isSubmitting}
          >
            {isSubmitting ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <CheckCircle2 className="h-4 w-4 mr-2" />
            )}
            Authorize
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
