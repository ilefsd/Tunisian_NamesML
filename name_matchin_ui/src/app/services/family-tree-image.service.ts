import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { map } from 'rxjs/operators';

export interface FamilyTreeImageResponse {
  imageUrl: string;
}

@Injectable({
  providedIn: 'root'
})
export class FamilyTreeImageService {

  private webhookUrl = '/webhook/6a89732f-408f-4f27-8607-f0a028107780';

  constructor(private http: HttpClient) { }

  getFamilyTreeImage(familyTreeData: any): Observable<string> {
    // The response from the n8n workflow should be a JSON object that contains the image URL.
    // Based on the user's description, the final output is a Google Drive link.
    // The n8n workflow should be configured to return a JSON object like:
    // { "imageUrl": "https://your-google-drive-link..." }
    // This service assumes the response has a property named `imageUrl`.

    return this.http.post<FamilyTreeImageResponse>(this.webhookUrl, familyTreeData).pipe(
      map(response => response.imageUrl)
    );
  }
}
