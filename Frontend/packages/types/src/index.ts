// @myai-workspace/types - shared type definitions
export interface BaseEntity {
  id: string | number;
  createdAt?: string;
  updatedAt?: string;
}

export interface User extends BaseEntity {
  username: string;
  email: string;
  avatar?: string;
  status: 'active' | 'inactive';
}

export interface Role extends BaseEntity {
  name: string;
  description: string;
  permissions: string[];
}

export interface Permission extends BaseEntity {
  name: string;
  code: string;
  type: 'menu' | 'button' | 'api';
}
