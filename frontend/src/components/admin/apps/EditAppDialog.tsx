import { useEffect } from 'react';
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
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { useAdminStore } from '@/stores/adminStore';
import { toast } from 'sonner';
import type { AdminAppDetail, AdminUpdateAppRequest } from '@/lib/auth-client';

const editAppSchema = z.object({
  name: z.string().min(1, 'Name is required').max(100, 'Name must be 100 characters or less'),
  code: z.string()
    .min(1, 'Code is required')
    .max(50, 'Code must be 50 characters or less')
    .regex(/^[a-z0-9_-]+$/, 'Code must contain only lowercase letters, numbers, hyphens, and underscores'),
});

type EditAppFormValues = z.infer<typeof editAppSchema>;

interface EditAppDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  app: AdminAppDetail;
  onSuccess?: () => void;
}

export function EditAppDialog({
  open,
  onOpenChange,
  app,
  onSuccess,
}: EditAppDialogProps) {
  const { updateApp, isLoading } = useAdminStore();

  const form = useForm<EditAppFormValues>({
    resolver: zodResolver(editAppSchema),
    defaultValues: {
      name: app.name,
      code: app.code,
    },
  });

  // Reset form when app changes
  useEffect(() => {
    form.reset({
      name: app.name,
      code: app.code,
    });
  }, [app, form]);

  const onSubmit = async (values: EditAppFormValues) => {
    try {
      const updateData: AdminUpdateAppRequest = {};
      
      // Only include changed fields
      if (values.name !== app.name) {
        updateData.name = values.name;
      }
      if (values.code !== app.code) {
        updateData.code = values.code;
      }

      // Check if any changes were made
      if (Object.keys(updateData).length === 0) {
        toast.info('No changes to save');
        onOpenChange(false);
        return;
      }

      await updateApp(app.id, updateData);
      toast.success('Application updated successfully');
      onOpenChange(false);
      onSuccess?.();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to update application');
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Edit Application</DialogTitle>
          <DialogDescription>
            Update application name and code.
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
                    <Input placeholder="My Application" {...field} />
                  </FormControl>
                  <FormDescription>
                    A human-readable name for the application
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="code"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Code</FormLabel>
                  <FormControl>
                    <Input placeholder="my-app" {...field} />
                  </FormControl>
                  <FormDescription>
                    A unique identifier (lowercase, numbers, hyphens, underscores)
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isLoading}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={isLoading}>
                {isLoading ? 'Saving...' : 'Save Changes'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
