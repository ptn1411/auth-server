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
import type { ApiKeyResponse, ApiKeyWithSecretResponse } from '@/lib/auth-client';

const API_KEY_SCOPES = [
  { id: 'read:users', label: 'Read Users' },
  { id: 'write:users', label: 'Write Users' },
  { id: 'read:roles', label: 'Read Roles' },
  { id: 'write:roles', label: 'Write Roles' },
  { id: 'read:permissions', label: 'Read Permissions' },
  { id: 'write:permissions', label: 'Write Permissions' },
];

const apiKeySchema = z.object({
  name: z
    .string()
    .min(2, 'Name must be at least 2 characters')
    .max(100, 'Name must be at most 100 characters'),
  scopes: z.array(z.string()).optional(),
  expires_at: z.string().optional(),
  is_active: z.boolean().optional(),
});

type ApiKeyFormData = z.infer<typeof apiKeySchema>;

interface ApiKeyDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: string;
  apiKey?: ApiKeyResponse;
  onApiKeyCreated?: (apiKey: ApiKeyWithSecretResponse) => void;
  onApiKeyUpdated?: () => void;
}

export function ApiKeyDialog({
  open,
  onOpenChange,
  appId,
  apiKey,
  onApiKeyCreated,
  onApiKeyUpdated,
}: ApiKeyDialogProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { createApiKey, updateApiKey } = useAppsStore();
  const isEditing = !!apiKey;

  const form = useForm<ApiKeyFormData>({
    resolver: zodResolver(apiKeySchema),
    defaultValues: {
      name: '',
      scopes: [],
      expires_at: '',
      is_active: true,
    },
  });

  useEffect(() => {
    if (apiKey) {
      form.reset({
        name: apiKey.name,
        scopes: apiKey.scopes,
        expires_at: apiKey.expires_at ? apiKey.expires_at.split('T')[0] : '',
        is_active: apiKey.is_active,
      });
    } else {
      form.reset({
        name: '',
        scopes: [],
        expires_at: '',
        is_active: true,
      });
    }
  }, [apiKey, form]);

  const onSubmit = async (data: ApiKeyFormData) => {
    setIsSubmitting(true);
    try {
      if (isEditing && apiKey) {
        await updateApiKey(appId, apiKey.id, {
          name: data.name,
          scopes: data.scopes,
          is_active: data.is_active,
        });
        toast.success('API key updated successfully');
        onApiKeyUpdated?.();
      } else {
        const newApiKey = await createApiKey(appId, {
          name: data.name,
          scopes: data.scopes,
          expires_at: data.expires_at || undefined,
        });
        onApiKeyCreated?.(newApiKey);
      }
      form.reset();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : `Failed to ${isEditing ? 'update' : 'create'} API key`);
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
          <DialogTitle>{isEditing ? 'Edit API Key' : 'Create API Key'}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? 'Update the API key configuration.'
              : 'Create a new API key for programmatic access.'}
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Production API Key"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    A descriptive name for this API key
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="scopes"
              render={() => (
                <FormItem>
                  <FormLabel>Scopes (Optional)</FormLabel>
                  <FormDescription>
                    Leave empty for full access, or select specific scopes
                  </FormDescription>
                  <div className="grid grid-cols-2 gap-2 mt-2">
                    {API_KEY_SCOPES.map((scope) => (
                      <FormField
                        key={scope.id}
                        control={form.control}
                        name="scopes"
                        render={({ field }) => (
                          <FormItem className="flex items-center space-x-2 space-y-0">
                            <FormControl>
                              <Checkbox
                                checked={field.value?.includes(scope.id)}
                                onCheckedChange={(checked) => {
                                  const newValue = checked
                                    ? [...(field.value || []), scope.id]
                                    : field.value?.filter((v) => v !== scope.id) || [];
                                  field.onChange(newValue);
                                }}
                                disabled={isSubmitting}
                              />
                            </FormControl>
                            <FormLabel className="text-sm font-normal cursor-pointer">
                              {scope.label}
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

            {!isEditing && (
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
            )}

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
                  'Update API Key'
                ) : (
                  'Create API Key'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
