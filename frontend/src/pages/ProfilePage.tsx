import { ProfileCard } from '@/components/profile/ProfileCard';
import { UpdateProfileForm } from '@/components/profile/UpdateProfileForm';
import { ChangePasswordForm } from '@/components/profile/ChangePasswordForm';

export function ProfilePage() {
  return (
    <div className="space-y-4 sm:space-y-6">
      <div>
        <h1 className="text-2xl sm:text-3xl font-bold">Profile</h1>
        <p className="text-sm sm:text-base text-muted-foreground">
          Manage your account settings
        </p>
      </div>

      <ProfileCard />
      
      <div className="grid gap-4 sm:gap-6 md:grid-cols-2">
        <UpdateProfileForm />
        <ChangePasswordForm />
      </div>
    </div>
  );
}
