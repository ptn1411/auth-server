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
import type { SearchUsersParams } from '@/lib/auth-client';

const searchSchema = z.object({
  email: z.string().optional(),
  is_active: z.string().optional(),
  is_system_admin: z.string().optional(),
});

type SearchFormValues = z.infer<typeof searchSchema>;

interface UserSearchFormProps {
  onSearch: (params: SearchUsersParams) => void;
  onClear: () => void;
  isLoading?: boolean;
}

export function UserSearchForm({ onSearch, onClear, isLoading }: UserSearchFormProps) {
  const form = useForm<SearchFormValues>({
    resolver: zodResolver(searchSchema),
    defaultValues: {
      email: '',
      is_active: 'all',
      is_system_admin: 'all',
    },
  });

  const onSubmit = (values: SearchFormValues) => {
    const params: SearchUsersParams = {};
    
    if (values.email && values.email.trim()) {
      params.email = values.email.trim();
    }
    
    if (values.is_active && values.is_active !== 'all') {
      params.is_active = values.is_active === 'true';
    }
    
    if (values.is_system_admin && values.is_system_admin !== 'all') {
      params.is_system_admin = values.is_system_admin === 'true';
    }
    
    onSearch(params);
  };

  const handleClear = () => {
    form.reset({
      email: '',
      is_active: 'all',
      is_system_admin: 'all',
    });
    onClear();
  };

  const hasFilters = form.watch('email') || 
    form.watch('is_active') !== 'all' || 
    form.watch('is_system_admin') !== 'all';

  return (
    <Card>
      <CardContent className="pt-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-wrap gap-4 items-end">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem className="flex-1 min-w-[200px]">
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Search by email..."
                      {...field}
                    />
                  </FormControl>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="is_active"
              render={({ field }) => (
                <FormItem className="w-[150px]">
                  <FormLabel>Status</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="All" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="all">All</SelectItem>
                      <SelectItem value="true">Active</SelectItem>
                      <SelectItem value="false">Inactive</SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="is_system_admin"
              render={({ field }) => (
                <FormItem className="w-[150px]">
                  <FormLabel>Role</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="All" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="all">All</SelectItem>
                      <SelectItem value="true">Admin</SelectItem>
                      <SelectItem value="false">User</SelectItem>
                    </SelectContent>
                  </Select>
                </FormItem>
              )}
            />

            <div className="flex gap-2">
              <Button type="submit" disabled={isLoading}>
                <Search className="h-4 w-4 mr-2" />
                Search
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
