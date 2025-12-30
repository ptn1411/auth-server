import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
} from '@/components/ui/form';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Search, X } from 'lucide-react';

const filterSchema = z.object({
  user_id: z.string().optional(),
  action: z.string().optional(),
});

type FilterFormValues = z.infer<typeof filterSchema>;

export interface AuditLogFilterParams {
  user_id?: string;
  action?: string;
}

interface AuditLogFiltersProps {
  onFilter: (params: AuditLogFilterParams) => void;
  onClear: () => void;
  isLoading?: boolean;
}

const ACTION_TYPES = [
  { value: 'LOGIN', label: 'Login' },
  { value: 'LOGOUT', label: 'Logout' },
  { value: 'REGISTER', label: 'Register' },
  { value: 'PASSWORD_CHANGE', label: 'Password Change' },
  { value: 'PASSWORD_RESET', label: 'Password Reset' },
  { value: 'EMAIL_VERIFICATION', label: 'Email Verification' },
  { value: 'MFA_ENABLE', label: 'MFA Enable' },
  { value: 'MFA_DISABLE', label: 'MFA Disable' },
  { value: 'MFA_VERIFY', label: 'MFA Verify' },
  { value: 'SESSION_REVOKE', label: 'Session Revoke' },
  { value: 'PROFILE_UPDATE', label: 'Profile Update' },
];

export function AuditLogFilters({ onFilter, onClear, isLoading }: AuditLogFiltersProps) {
  const form = useForm<FilterFormValues>({
    resolver: zodResolver(filterSchema),
    defaultValues: {
      user_id: '',
      action: 'all',
    },
  });

  const onSubmit = (values: FilterFormValues) => {
    const params: AuditLogFilterParams = {};
    
    if (values.user_id && values.user_id.trim()) {
      params.user_id = values.user_id.trim();
    }
    
    if (values.action && values.action !== 'all') {
      params.action = values.action;
    }
    
    onFilter(params);
  };

  const handleClear = () => {
    form.reset({
      user_id: '',
      action: 'all',
    });
    onClear();
  };

  const hasFilters = form.watch('user_id') || form.watch('action') !== 'all';

  return (
    <Card>
      <CardContent className="pt-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-wrap gap-4 items-end">
            <FormField
              control={form.control}
              name="user_id"
              render={({ field }) => (
                <FormItem className="flex-1 min-w-[200px]">
                  <FormLabel>User ID</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Filter by user ID..."
                      {...field}
                    />
                  </FormControl>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="action"
              render={({ field }) => (
                <FormItem className="w-[200px]">
                  <FormLabel>Action Type</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="All Actions" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="all">All Actions</SelectItem>
                      {ACTION_TYPES.map((action) => (
                        <SelectItem key={action.value} value={action.value}>
                          {action.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <div className="flex gap-2">
              <Button type="submit" disabled={isLoading}>
                <Search className="h-4 w-4 mr-2" />
                Filter
              </Button>
              {hasFilters && (
                <Button type="button" variant="outline" onClick={handleClear}>
                  <X className="h-4 w-4 mr-2" />
                  Clear
                </Button>
              )}
            </div>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
