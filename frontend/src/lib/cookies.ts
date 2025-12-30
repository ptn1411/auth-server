// Cookie utility functions for refresh token storage
// Access token is stored in memory (Zustand) for security

const REFRESH_TOKEN_KEY = 'refresh_token';
const REFRESH_TOKEN_EXPIRY_DAYS = 7;

interface CookieOptions {
  expires?: number; // days
  path?: string;
  secure?: boolean;
  sameSite?: 'strict' | 'lax' | 'none';
}

function setCookie(name: string, value: string, options: CookieOptions = {}): void {
  const {
    expires = 7,
    path = '/',
    secure = window.location.protocol === 'https:',
    sameSite = 'lax',
  } = options;

  const expiryDate = new Date();
  expiryDate.setTime(expiryDate.getTime() + expires * 24 * 60 * 60 * 1000);

  let cookieString = `${encodeURIComponent(name)}=${encodeURIComponent(value)}`;
  cookieString += `; expires=${expiryDate.toUTCString()}`;
  cookieString += `; path=${path}`;
  cookieString += `; SameSite=${sameSite}`;
  
  if (secure) {
    cookieString += '; Secure';
  }

  document.cookie = cookieString;
}

function getCookie(name: string): string | null {
  const nameEQ = encodeURIComponent(name) + '=';
  const cookies = document.cookie.split(';');
  
  for (let cookie of cookies) {
    cookie = cookie.trim();
    if (cookie.indexOf(nameEQ) === 0) {
      return decodeURIComponent(cookie.substring(nameEQ.length));
    }
  }
  
  return null;
}

function deleteCookie(name: string, path: string = '/'): void {
  document.cookie = `${encodeURIComponent(name)}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=${path}`;
}

// Refresh token functions (stored in cookie)
export function setRefreshToken(token: string): void {
  setCookie(REFRESH_TOKEN_KEY, token, { expires: REFRESH_TOKEN_EXPIRY_DAYS });
}

export function getRefreshToken(): string | null {
  return getCookie(REFRESH_TOKEN_KEY);
}

export function clearRefreshToken(): void {
  deleteCookie(REFRESH_TOKEN_KEY);
}
