import { useState, useEffect } from 'react';
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
import { Checkbox } from '@/components/ui/checkbox';
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Loader2 } from 'lucide-react';
import type { WebhookResponse, WebhookWithSecretResponse } from '@/lib/auth-client';

const WEBHOOK_EVENTS = [
  { id: 'user.created', label: 'User Created' },
  { id: 'user.updated', label: 'User Updated' },
  { id: 'user.deleted', label: 'User Deleted' },
  { id: 'user.login', label: 'User Login' },
  { id: 'user.logout', label: 'User Logout' },
  { id: 'user.password_changed', label: 'Password Changed' },
  { id: 'user.mfa_enabled', label: 'MFA Enabled' },
  { id: 'user.mfa_disabled', label: 'MFA Disabled' },
];

const webhookSchema = z.object({
  url: z.string().url('Please enter a valid URL'),
  events: z.array(z.string()).min(1, 'Select at least one event'),
  is_active: z.boolean().optional(),
});

type WebhookFormData = z.infer<typeof webhookSchema>;

interface WebhookDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: string;
  webhook?: WebhookResponse;
  onWebhookCreated?: (webhook: WebhookWithSecretResponse) => void;
  onWebhookUpdated?: () => void;
}

export function WebhookDialog({
  open,
  onOpenChange,
  appId,
  webhook,
  onWebhookCreated,
  onWebhookUpdated,
}: WebhookDialogProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { createWebhook, updateWebhook } = useAppsStore();
  const isEditing = !!webhook;

  const form = useForm<WebhookFormData>({
    resolver: zodResolver(webhookSchema),
    defaultValues: {
      url: '',
      events: [],
      is_active: true,
    },
  });

  useEffect(() => {
    if (webhook) {
      form.reset({
        url: webhook.url,
        events: webhook.events,
        is_active: webhook.is_active,
      });
    } else {
      form.reset({
        url: '',
        events: [],
        is_active: true,
      });
    }
  }, [webhook, form]);

  const onSubmit = async (data: WebhookFormData) => {
    setIsSubmitting(true);
    try {
      if (isEditing && webhook) {
        await updateWebhook(appId, webhook.id, {
          url: data.url,
          events: data.events,
          is_active: data.is_active,
        });
        toast.success('Webhook updated successfully');
        onWebhookUpdated?.();
      } else {
        const newWebhook = await createWebhook(appId, {
          url: data.url,
          events: data.events,
        });
        onWebhookCreated?.(newWebhook);
      }
      form.reset();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : `Failed to ${isEditing ? 'update' : 'create'} webhook`);
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
          <DialogTitle>{isEditing ? 'Edit Webhook' : 'Create Webhook'}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? 'Update the webhook configuration.'
              : 'Create a new webhook to receive event notifications.'}
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="url"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Webhook URL</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="https://example.com/webhook"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    The URL that will receive webhook events
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="events"
              render={() => (
                <FormItem>
                  <FormLabel>Events</FormLabel>
                  <FormDescription>
                    Select the events you want to receive
                  </FormDescription>
                  <div className="grid grid-cols-2 gap-2 mt-2">
                    {WEBHOOK_EVENTS.map((event) => (
                      <FormField
                        key={event.id}
                        control={form.control}
                        name="events"
                        render={({ field }) => (
                          <FormItem className="flex items-center space-x-2 space-y-0">
                            <FormControl>
                              <Checkbox
                                checked={field.value?.includes(event.id)}
                                onCheckedChange={(checked) => {
                                  const newValue = checked
                                    ? [...(field.value || []), event.id]
                                    : field.value?.filter((v) => v !== event.id) || [];
                                  field.onChange(newValue);
                                }}
                                disabled={isSubmitting}
                              />
                            </FormControl>
                            <FormLabel className="text-sm font-normal cursor-pointer">
                              {event.label}
                            </FormLabel>
                          </FormItem>
                        )}
                      />
                    ))}
                  </div>
                  <FormMessage />
                </FormItem>
              )}
            />

            {isEditing && (
              <FormField
                control={form.control}
                name="is_active"
                render={({ field }) => (
                  <FormItem className="flex items-center space-x-2 space-y-0">
                    <FormControl>
                      <Checkbox
                        checked={field.value}
                        onCheckedChange={field.onChange}
                        disabled={isSubmitting}
                      />
                    </FormControl>
                    <FormLabel className="font-normal cursor-pointer">
                      Active
                    </FormLabel>
                  </FormItem>
                )}
              />
            )}

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
                    {isEditing ? 'Updating...' : 'Creating...'}
                  </>
                ) : isEditing ? (
                  'Update Webhook'
                ) : (
                  'Create Webhook'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
