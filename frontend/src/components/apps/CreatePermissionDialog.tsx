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
import { useAppsStore } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Loader2 } from 'lucide-react';

const createPermissionSchema = z.object({
  code: z
    .string()
    .min(2, 'Code must be at least 2 characters')
    .max(50, 'Code must be at most 50 characters')
    .regex(/^[a-z0-9_:.-]+$/, 'Code must be lowercase alphanumeric with underscores, colons, dots, or dashes'),
});

type CreatePermissionFormData = z.infer<typeof createPermissionSchema>;

interface CreatePermissionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: string;
  onPermissionCreated?: () => void;
}

export function CreatePermissionDialog({
  open,
  onOpenChange,
  appId,
  onPermissionCreated,
}: CreatePermissionDialogProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { createPermission } = useAppsStore();

  const form = useForm<CreatePermissionFormData>({
    resolver: zodResolver(createPermissionSchema),
    defaultValues: {
      code: '',
    },
  });

  const onSubmit = async (data: CreatePermissionFormData) => {
    setIsSubmitting(true);
    try {
      await createPermission(appId, data.code);
      toast.success('Permission created successfully');
      form.reset();
      onPermissionCreated?.();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create permission');
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
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Permission</DialogTitle>
          <DialogDescription>
            Create a new permission to define granular access controls.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="code"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Permission Code</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="read:users"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    A unique code for this permission (e.g., read:users, write:posts)
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
                  'Create Permission'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
