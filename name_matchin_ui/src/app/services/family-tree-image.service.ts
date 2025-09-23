import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class FamilyTreeImageService {

  private webhookUrl = '/webhook-proxy/webhook/6a89732f-408f-4f27-8607-f0a028107780';

  constructor(private http: HttpClient) { }

  /**
   * Sends Neo4j JSON data to the webhook to generate a family tree image.
   * @param neo4jData The Neo4j JSON object.
   * @returns An Observable that resolves to an image Blob.
   */
  generateTreeImage(neo4jData: any): Observable<Blob> {
    const headers = new HttpHeaders({ 'Content-Type': 'application/json' });
    return this.http.post(this.webhookUrl, neo4jData, {
      headers: headers,
      responseType: 'blob'
    });
  }
}
