import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';

/**
 * OAuth Callback Page
 * 
 * This page handles the OAuth redirect callback.
 * It extracts the authorization code/error from URL params
 * and sends them back to the opener window via postMessage.
 */
export function OAuthCallbackPage() {
  const [searchParams] = useSearchParams();

  useEffect(() => {
    // Extract params from URL
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');
    const errorDescription = searchParams.get('error_description');

    // Build message
    const message = {
      type: 'oauth_callback',
      code,
      state,
      error,
      error_description: errorDescription,
    };

    // Send to opener window
    if (window.opener) {
      window.opener.postMessage(message, '*');
      // Close popup after sending message
      setTimeout(() => window.close(), 100);
    }
  }, [searchParams]);

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
        <p className="text-muted-foreground">Processing authorization...</p>
      </div>
    </div>
  );
}
