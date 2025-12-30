import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Copy, Check, Eye, EyeOff, AlertTriangle } from 'lucide-react';
import { toast } from 'sonner';

interface SecretDisplayProps {
  title: string;
  description?: string;
  secret: string;
  warning?: string;
  onClose?: () => void;
}

export function SecretDisplay({
  title,
  description,
  secret,
  warning = 'This secret will only be shown once. Please save it securely.',
  onClose,
}: SecretDisplayProps) {
  const [copied, setCopied] = useState(false);
  const [visible, setVisible] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(secret);
      setCopied(true);
      toast.success('Secret copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error('Failed to copy secret');
    }
  };

  const toggleVisibility = () => {
    setVisible(!visible);
  };

  const maskedSecret = secret.replace(/./g, '•');

  return (
    <Card className="border-amber-200 bg-amber-50 dark:border-amber-900 dark:bg-amber-950">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-amber-800 dark:text-amber-200">
          <AlertTriangle className="h-5 w-5" />
          {title}
        </CardTitle>
        {description && (
          <CardDescription className="text-amber-700 dark:text-amber-300">
            {description}
          </CardDescription>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-2">
          <code className="flex-1 rounded-md bg-white p-3 font-mono text-sm dark:bg-gray-900 break-all">
            {visible ? secret : maskedSecret}
          </code>
          <div className="flex flex-col gap-1">
            <Button
              variant="outline"
              size="icon-sm"
              onClick={toggleVisibility}
              title={visible ? 'Hide secret' : 'Show secret'}
            >
              {visible ? (
                <EyeOff className="h-4 w-4" />
              ) : (
                <Eye className="h-4 w-4" />
              )}
            </Button>
            <Button
              variant="outline"
              size="icon-sm"
              onClick={handleCopy}
              title="Copy to clipboard"
            >
              {copied ? (
                <Check className="h-4 w-4 text-green-600" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>
        
        {warning && (
          <p className="text-sm text-amber-700 dark:text-amber-300">
            {warning}
          </p>
        )}
        
        {onClose && (
          <div className="flex justify-end">
            <Button onClick={onClose}>
              I've saved the secret
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
