import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';
import {UpdateUser, UserResponse} from '../models/user.model';

@Injectable({
  providedIn: 'root'
})
export class UserService {
  private apiUrl = '/api/users';

  constructor(private http: HttpClient) { }

  getUsers(): Observable<UserResponse[]> {
    return this.http.get<UserResponse[]>(this.apiUrl, this.getHttpOptions());
  }

  addUser(user: any): Observable<UserResponse> {
    return this.http.post<UserResponse>(this.apiUrl, user, this.getHttpOptions());
  }

  updateUser(user: any): Observable<any> {
    const updateUser: UpdateUser = {
      email: user.email,
      password: user.password
    };
    return this.http.put(`${this.apiUrl}/${user.id}`, updateUser, this.getHttpOptions());
  }

  deleteUser(id: string): Observable<any> {
    return this.http.delete(`${this.apiUrl}/${id}`, this.getHttpOptions());
  }

  private getHttpOptions() {
    const token = localStorage.getItem('token');
    return {
      headers: new HttpHeaders({
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      })
    };
  }
}
