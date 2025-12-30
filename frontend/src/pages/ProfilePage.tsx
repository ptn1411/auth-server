import { ProfileCard } from '@/components/profile/ProfileCard';
import { UpdateProfileForm } from '@/components/profile/UpdateProfileForm';
import { ChangePasswordForm } from '@/components/profile/ChangePasswordForm';

export function ProfilePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Profile</h1>
        <p className="text-muted-foreground">
          Manage your account settings
        </p>
      </div>

      <ProfileCard />
      
      <div className="grid gap-6 md:grid-cols-2">
        <UpdateProfileForm />
        <ChangePasswordForm />
      </div>
    </div>
  );
}
