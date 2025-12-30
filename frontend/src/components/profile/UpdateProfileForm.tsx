import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuthStore } from '@/stores/authStore';
import { authClient, AuthServerError } from '@/lib/auth-client';
import { toast } from 'sonner';
import { Loader2, Mail } from 'lucide-react';

const updateProfileSchema = z.object({
  email: z.string().email('Invalid email address'),
});

type UpdateProfileFormData = z.infer<typeof updateProfileSchema>;

export function UpdateProfileForm() {
  const { user, refreshUser } = useAuthStore();
  const [isLoading, setIsLoading] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors, isDirty },
  } = useForm<UpdateProfileFormData>({
    resolver: zodResolver(updateProfileSchema),
    defaultValues: {
      email: user?.email || '',
    },
  });

  const onSubmit = async (data: UpdateProfileFormData) => {
    if (!isDirty) return;
    
    setIsLoading(true);
    try {
      await authClient.updateProfile({ email: data.email });
      await refreshUser();
      toast.success('Profile updated successfully');
    } catch (error) {
      if (error instanceof AuthServerError) {
        toast.error(error.message);
      } else {
        toast.error('Failed to update profile');
      }
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Mail className="h-5 w-5" />
          Update Email
        </CardTitle>
        <CardDescription>Change your email address</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              placeholder="Enter your email"
              {...register('email')}
              aria-invalid={!!errors.email}
            />
            {errors.email && (
              <p className="text-sm text-destructive">{errors.email.message}</p>
            )}
          </div>
          
          <Button type="submit" disabled={isLoading || !isDirty}>
            {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Update Email
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
