import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';
import { Copy, Check, Download } from 'lucide-react';

interface BackupCodesProps {
  codes: string[];
}

export function BackupCodes({ codes }: BackupCodesProps) {
  const [copied, setCopied] = useState(false);

  const copyAllCodes = async () => {
    try {
      await navigator.clipboard.writeText(codes.join('\n'));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      toast.success('Backup codes copied to clipboard');
    } catch {
      toast.error('Failed to copy codes');
    }
  };

  const downloadCodes = () => {
    const content = `Auth Server Backup Codes
========================
Generated: ${new Date().toLocaleString()}

Keep these codes in a safe place. Each code can only be used once.

${codes.join('\n')}

========================
If you lose access to your authenticator app, you can use one of these codes to sign in.
`;
    
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'backup-codes.txt';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    toast.success('Backup codes downloaded');
  };

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-2 p-4 bg-muted rounded-lg">
        {codes.map((code, index) => (
          <code
            key={index}
            className="px-3 py-2 bg-background rounded text-center font-mono text-sm"
          >
            {code}
          </code>
        ))}
      </div>

      <div className="flex gap-2 justify-center">
        <Button variant="outline" size="sm" onClick={copyAllCodes}>
          {copied ? (
            <>
              <Check className="h-4 w-4 mr-2" />
              Copied!
            </>
          ) : (
            <>
              <Copy className="h-4 w-4 mr-2" />
              Copy All
            </>
          )}
        </Button>
        <Button variant="outline" size="sm" onClick={downloadCodes}>
          <Download className="h-4 w-4 mr-2" />
          Download
        </Button>
      </div>

      <p className="text-xs text-muted-foreground text-center">
        Each code can only be used once. Store them securely.
      </p>
    </div>
  );
}
