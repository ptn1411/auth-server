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
import { useAppsStore, type AppWithSecret } from '@/stores/appsStore';
import { toast } from 'sonner';
import { Loader2 } from 'lucide-react';

const createAppSchema = z.object({
  code: z
    .string()
    .min(2, 'Code must be at least 2 characters')
    .max(50, 'Code must be at most 50 characters')
    .regex(/^[a-z0-9_-]+$/, 'Code must be lowercase alphanumeric with dashes or underscores'),
  name: z
    .string()
    .min(2, 'Name must be at least 2 characters')
    .max(100, 'Name must be at most 100 characters'),
});

type CreateAppFormData = z.infer<typeof createAppSchema>;

interface CreateAppDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAppCreated: (app: AppWithSecret) => void;
}

export function CreateAppDialog({ open, onOpenChange, onAppCreated }: CreateAppDialogProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { createApp } = useAppsStore();

  const form = useForm<CreateAppFormData>({
    resolver: zodResolver(createAppSchema),
    defaultValues: {
      code: '',
      name: '',
    },
  });

  const onSubmit = async (data: CreateAppFormData) => {
    setIsSubmitting(true);
    try {
      const app = await createApp(data);
      toast.success('App created successfully');
      form.reset();
      onAppCreated(app);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create app');
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
          <DialogTitle>Create New App</DialogTitle>
          <DialogDescription>
            Create a new application to integrate with the auth system.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="code"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>App Code</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="my-app"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    Unique identifier for your app (lowercase, alphanumeric, dashes, underscores)
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>App Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="My Application"
                      {...field}
                      disabled={isSubmitting}
                    />
                  </FormControl>
                  <FormDescription>
                    Display name for your application
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
                  'Create App'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
