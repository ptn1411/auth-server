import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import { Loader2 } from 'lucide-react';

const ipRuleSchema = z.object({
  ip_address: z
    .string()
    .min(1, 'IP address is required')
    .regex(
      /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
      'Please enter a valid IPv4 address'
    ),
  ip_range: z.string().optional(),
  rule_type: z.enum(['whitelist', 'blacklist']),
  reason: z.string().max(255, 'Reason must be at most 255 characters').optional(),
  expires_at: z.string().optional(),
});

type IpRuleFormData = z.infer<typeof ipRuleSchema>;

interface CreateIpRuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onIpRuleCreated?: () => void;
}

export function CreateIpRuleDialog({
  open,
  onOpenChange,
  onIpRuleCreated,
}: CreateIpRuleDialogProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { createIpRule } = useAdminStore();

  const form = useForm<IpRuleFormData>({
    resolver: zodResolver(ipRuleSchema),
    defaultValues: {
      ip_address: '',
      ip_range: '',
      rule_type: 'blacklist',
      reason: '',
      expires_at: '',
    },
  });

  const onSubmit = async (data: IpRuleFormData) => {
    setIsSubmitting(true);
    try {
      await createIpRule({
        ip_address: data.ip_address,
        ip_range: data.ip_range || undefined,
        rule_type: data.rule_type,
        reason: data.reason || undefined,
        expires_at: data.expires_at || undefined,
      });
      toast.success('IP rule created successfully');
      form.reset();
      onIpRuleCreated?.();
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create IP rule');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      form.reset();
    }
    onOpenChange(newOpen);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create Global IP Rule</DialogTitle>
          <DialogDescription>
            Create a new system-wide IP rule to whitelist or blacklist an IP address.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="ip_address"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>IP Address</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="192.168.1.1"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    The IPv4 address to apply the rule to
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="ip_range"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>IP Range (Optional)</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="/24"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    CIDR notation for IP range (e.g., /24 for 256 addresses)
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="rule_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Rule Type</FormLabel>
                  <Select
                    onValueChange={field.onChange}
                    defaultValue={field.value}
                    disabled={isSubmitting}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select rule type" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="blacklist">Blacklist (Block)</SelectItem>
                      <SelectItem value="whitelist">Whitelist (Allow)</SelectItem>
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    Whether to allow or block this IP address
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="reason"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Reason (Optional)</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Suspicious activity"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    A note explaining why this rule was created
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="expires_at"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Expiration Date (Optional)</FormLabel>
                  <FormControl>
                    <Input
                      type="date"
                      {...field}
                      disabled={isSubmitting}
                      min={new Date().toISOString().split('T')[0]}
                    />
                  </FormControl>
                  <FormDescription>
                    Leave empty for no expiration
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => handleOpenChange(false)}
                disabled={isSubmitting}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Creating...
                  </>
                ) : (
                  'Create IP Rule'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
