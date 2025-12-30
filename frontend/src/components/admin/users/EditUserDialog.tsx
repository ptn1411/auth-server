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
import { Checkbox } from '@/components/ui/checkbox';
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
import type { AdminUserDetail, AdminUpdateUserRequest } from '@/lib/auth-client';

const editUserSchema = z.object({
  email: z.string().email('Invalid email address'),
  is_active: z.boolean(),
  email_verified: z.boolean(),
  is_system_admin: z.boolean(),
});

type EditUserFormValues = z.infer<typeof editUserSchema>;

interface EditUserDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  user: AdminUserDetail;
  onSuccess?: () => void;
}

export function EditUserDialog({
  open,
  onOpenChange,
  user,
  onSuccess,
}: EditUserDialogProps) {
  const { updateUser, isLoading } = useAdminStore();

  const form = useForm<EditUserFormValues>({
    resolver: zodResolver(editUserSchema),
    defaultValues: {
      email: user.email,
      is_active: user.is_active,
      email_verified: user.email_verified,
      is_system_admin: user.is_system_admin,
    },
  });

  // Reset form when user changes
  useEffect(() => {
    form.reset({
      email: user.email,
      is_active: user.is_active,
      email_verified: user.email_verified,
      is_system_admin: user.is_system_admin,
    });
  }, [user, form]);

  const onSubmit = async (values: EditUserFormValues) => {
    try {
      const updateData: AdminUpdateUserRequest = {};
      
      // Only include changed fields
      if (values.email !== user.email) {
        updateData.email = values.email;
      }
      if (values.is_active !== user.is_active) {
        updateData.is_active = values.is_active;
      }
      if (values.email_verified !== user.email_verified) {
        updateData.email_verified = values.email_verified;
      }
      if (values.is_system_admin !== user.is_system_admin) {
        updateData.is_system_admin = values.is_system_admin;
      }

      // Check if any changes were made
      if (Object.keys(updateData).length === 0) {
        toast.info('No changes to save');
        onOpenChange(false);
        return;
      }

      await updateUser(user.id, updateData);
      toast.success('User updated successfully');
      onOpenChange(false);
      onSuccess?.();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to update user');
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Edit User</DialogTitle>
          <DialogDescription>
            Update user information and permissions.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input placeholder="user@example.com" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="is_active"
              render={({ field }) => (
                <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                  <FormControl>
                    <Checkbox
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                  <div className="space-y-1 leading-none">
                    <FormLabel>Active</FormLabel>
                    <FormDescription>
                      User can log in and access the system
                    </FormDescription>
                  </div>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="email_verified"
              render={({ field }) => (
                <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                  <FormControl>
                    <Checkbox
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                  <div className="space-y-1 leading-none">
                    <FormLabel>Email Verified</FormLabel>
                    <FormDescription>
                      Mark the user's email as verified
                    </FormDescription>
                  </div>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="is_system_admin"
              render={({ field }) => (
                <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                  <FormControl>
                    <Checkbox
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                  <div className="space-y-1 leading-none">
                    <FormLabel>System Admin</FormLabel>
                    <FormDescription>
                      Grant full administrative access to the system
                    </FormDescription>
                  </div>
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
