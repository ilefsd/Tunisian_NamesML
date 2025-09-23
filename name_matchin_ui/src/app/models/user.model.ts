export interface UserResponse {
  id: string;
  email: string;
}

export interface UpdateUser {
  email?: string;
  password?: string;
}
