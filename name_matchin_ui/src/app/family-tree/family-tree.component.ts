import { Component, Inject, OnInit } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { Neo4jService } from '../services/neo4j.service.ts.service';
import { DomSanitizer, SafeResourceUrl } from '@angular/platform-browser';

export interface FamilyTreeModalData {
  identity: {
    first_name: string;
    father_name?: string;
    mother_name?: string;
  };
}

@Component({
  selector: 'app-family-tree',
  standalone: false,
  templateUrl: './family-tree.component.html',
  styleUrls: ['./family-tree.component.css']
})
export class FamilyTreeComponent implements OnInit {
  isLoading = true;
  error: string | null = null;
  browserUrl: SafeResourceUrl | null = null;

  constructor(
    public dialogRef: MatDialogRef<FamilyTreeComponent>,
    @Inject(MAT_DIALOG_DATA) public data: FamilyTreeModalData,
    private neo4jService: Neo4jService,
    private sanitizer: DomSanitizer
  ) {}

  ngOnInit(): void {
    this.loadFamilyTree();
  }

  loadFamilyTree(): void {
    this.isLoading = true;
    this.error = null;

    if (!this.data || !this.data.identity) {
      this.error = "No identity data provided to the modal.";
      this.isLoading = false;
      return;
    }

    const childName = this.data.identity.first_name;
    const fatherName = this.data.identity.father_name || '';
    const motherName = this.data.identity.mother_name || '';

    if (!childName) {
      this.error = "Child's first name is required to fetch the family tree.";
      this.isLoading = false;
      return;
    }

    try {
      const url = this.neo4jService.getFamilyTreeUrl(
        childName,
        fatherName,
        motherName
      );
      this.browserUrl = this.sanitizer.bypassSecurityTrustResourceUrl(url);
      this.isLoading = false;
    } catch (err) {
      console.error('Error getting family tree URL:', err);
      this.error = `Failed to get family tree URL. ${err instanceof Error ? err.message : String(err)}`;
      this.isLoading = false;
    }
  }

  close(): void {
    this.dialogRef.close();
  }
}
