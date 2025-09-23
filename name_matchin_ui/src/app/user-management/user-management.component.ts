import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { UserService } from '../services/user.service';
import { UserResponse } from '../models/user.model';

@Component({
  selector: 'app-user-management',
  standalone: false,
  templateUrl: './user-management.component.html',
  styleUrls: ['./user-management.component.css']
})
export class UserManagementComponent implements OnInit {
  users: UserResponse[] = [];
  userForm: FormGroup;
  editMode = false;
  userIdToUpdate: string | null = null;

  constructor(
    private fb: FormBuilder,
    private userService: UserService
  ) {
    this.userForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', Validators.required]
    });
  }

  ngOnInit(): void {
    this.getUsers();
  }

  getUsers(): void {
    this.userService.getUsers().subscribe(users => this.users = users);
  }

  onFormSubmit(): void {
    if (this.userForm.invalid) {
      return;
    }

    if (this.editMode) {
      this.updateUser();
    } else {
      this.addUser();
    }
  }

  addUser(): void {
    this.userService.addUser(this.userForm.value).subscribe(newUser => {
      this.users.push(newUser);
      this.userForm.reset();
    });
  }

  updateUser(): void {
    if (!this.userIdToUpdate) {
      return;
    }

    const formValue = this.userForm.value;
    const payload: any = {
      id: this.userIdToUpdate,
      email: formValue.email,
    };

    if (formValue.password) {
      payload.password = formValue.password;
    }

    this.userService.updateUser(payload).subscribe(() => {
      this.getUsers();
      this.cancelEdit();
    });
  }

  editUser(user: UserResponse): void {
    this.editMode = true;
    this.userIdToUpdate = user.id;
    this.userForm.patchValue({
      email: user.email
    });
    this.userForm.get('password')?.clearValidators();
    this.userForm.get('password')?.updateValueAndValidity();
  }

  cancelEdit(): void {
    this.editMode = false;
    this.userIdToUpdate = null;
    this.userForm.reset();
    this.userForm.get('password')?.setValidators(Validators.required);
    this.userForm.get('password')?.updateValueAndValidity();
  }

  deleteUser(id: string): void {
    this.userService.deleteUser(id).subscribe(() => {
      this.users = this.users.filter(u => u.id !== id);
    });
  }
}
