export interface UserResponse {
  id: string;
  email: string;
  api_usage_count: number;
}

export interface UpdateUser {
  email?: string;
  password?: string;
}
