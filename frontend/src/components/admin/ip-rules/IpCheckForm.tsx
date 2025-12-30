import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import { Search, Loader2, CheckCircle, XCircle, Shield, ShieldOff } from 'lucide-react';
import type { IpCheckResponse } from '@/lib/auth-client';

const ipCheckSchema = z.object({
  ip_address: z
    .string()
    .min(1, 'IP address is required')
    .regex(
      /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      'Please enter a valid IPv4 address'
    ),
  app_id: z.string().optional(),
});

type IpCheckFormData = z.infer<typeof ipCheckSchema>;

export function IpCheckForm() {
  const [isChecking, setIsChecking] = useState(false);
  const [checkResult, setCheckResult] = useState<IpCheckResponse | null>(null);
  const { checkIp } = useAdminStore();

  const form = useForm<IpCheckFormData>({
    resolver: zodResolver(ipCheckSchema),
    defaultValues: {
      ip_address: '',
      app_id: '',
    },
  });

  const onSubmit = async (data: IpCheckFormData) => {
    setIsChecking(true);
    setCheckResult(null);
    try {
      const result = await checkIp(data.ip_address, data.app_id || undefined);
      setCheckResult(result);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to check IP');
    } finally {
      setIsChecking(false);
    }
  };

  const handleClear = () => {
    form.reset();
    setCheckResult(null);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Search className="h-5 w-5" />
          IP Check
        </CardTitle>
        <CardDescription>
          Check if an IP address is allowed or blocked by the system
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="flex gap-4">
              <FormField
                control={form.control}
                name="ip_address"
                render={({ field }) => (
                  <FormItem className="flex-1">
                    <FormLabel>IP Address</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="192.168.1.1"
                        {...field}
                        disabled={isChecking}
                      />
                    </FormControl>
                    <FormDescription>
                      Enter an IPv4 address to check
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="app_id"
                render={({ field }) => (
                  <FormItem className="flex-1">
                    <FormLabel>App ID (Optional)</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="app-uuid"
                        {...field}
                        disabled={isChecking}
                      />
                    </FormControl>
                    <FormDescription>
                      Check against app-specific rules
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="flex gap-2">
              <Button type="submit" disabled={isChecking}>
                {isChecking ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Checking...
                  </>
                ) : (
                  <>
                    <Search className="h-4 w-4 mr-2" />
                    Check IP
                  </>
                )}
              </Button>
              {checkResult && (
                <Button type="button" variant="outline" onClick={handleClear}>
                  Clear
                </Button>
              )}
            </div>
          </form>
        </Form>

        {/* Check Result */}
        {checkResult && (
          <div className="mt-6 p-4 border rounded-lg">
            <div className="flex items-center gap-3 mb-4">
              {checkResult.allowed ? (
                <>
                  <CheckCircle className="h-8 w-8 text-green-500" />
                  <div>
                    <h3 className="text-lg font-semibold text-green-700 dark:text-green-400">
                      IP Allowed
                    </h3>
                    <p className="text-sm text-muted-foreground">
                      This IP address is allowed to access the system
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <XCircle className="h-8 w-8 text-red-500" />
                  <div>
                    <h3 className="text-lg font-semibold text-red-700 dark:text-red-400">
                      IP Blocked
                    </h3>
                    <p className="text-sm text-muted-foreground">
                      This IP address is blocked from accessing the system
                    </p>
                  </div>
                </>
              )}
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">IP Address:</span>
                <code className="text-sm bg-muted px-2 py-1 rounded">
                  {form.getValues('ip_address')}
                </code>
              </div>

              {checkResult.rule_type && (
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Matching Rule:</span>
                  {checkResult.rule_type === 'whitelist' ? (
                    <Badge variant="secondary" className="flex items-center gap-1">
                      <Shield className="h-3 w-3" />
                      Whitelist
                    </Badge>
                  ) : (
                    <Badge variant="destructive" className="flex items-center gap-1">
                      <ShieldOff className="h-3 w-3" />
                      Blacklist
                    </Badge>
                  )}
                </div>
              )}

              {!checkResult.rule_type && checkResult.allowed && (
                <p className="text-sm text-muted-foreground">
                  No matching rule found. IP is allowed by default.
                </p>
              )}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
