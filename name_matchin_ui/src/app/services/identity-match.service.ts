import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

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

  constructor(private http: HttpClient) {}

  matchIdentity(input: InputIdentity): Observable<MatchResult[]> {
    return this.http.post<MatchResult[]>(this.apiUrl, input);
  }
}
