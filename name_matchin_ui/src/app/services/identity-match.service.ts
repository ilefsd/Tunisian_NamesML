import { Injectable } from '@angular/core';
import {HttpClient, HttpHeaders} from '@angular/common/http';
import { Observable } from 'rxjs';
import {AuthService} from './auth.service';

export interface InputIdentity {
  first_name:       string;
  last_name:        string;
  father_name:      string;
  grandfather_name: string;
  mother_last_name: string;
  mother_name:      string;
  dob:              [number, number, number] | null;
  sex:              number;
  place_of_birth:   string;
}

export interface FieldScore {
  field: string;
  score: number;
}

export interface MatchResult {
  matched_identity: {
    first_name:       string;
    last_name:        string;
    father_name:      string;
    grandfather_name: string;
    mother_last_name: string;
    mother_name:      string;
    dob:              [number, number, number];
    sex:              number;
    place_of_birth:   string;
  };
  total_score: number;
  breakdown:   FieldScore[];
}

@Injectable({ providedIn: 'root' })
export class IdentityMatchService {
  private apiUrl = '/match';

  constructor(private http: HttpClient, private authService: AuthService) { }

  matchIdentity(identity: InputIdentity): Observable<MatchResult[]> {
    const token = localStorage.getItem('token');
    const headers = new HttpHeaders().set('Authorization', `Bearer ${token}`);
    return this.http.post<MatchResult[]>(this.apiUrl, identity, { headers });
  }
}
