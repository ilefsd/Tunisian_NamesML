// src/app/family-tree/family-tree.component.ts

import {
  Component,
  ElementRef,
  Inject,
  OnInit,
  ViewChild,
  AfterViewInit,
  ChangeDetectorRef,
} from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { Neo4jService } from '../services/neo4j.service.ts.service';
import { FamilyPath } from '../models/family-path/family-path.module';
import { DataSet } from 'vis-data';
import {
  Network,
  type Node as VisNode,
  type Edge as VisEdge,
  Options,
} from 'vis-network/standalone';

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
  styleUrls: ['./family-tree.component.css'],
})
export class FamilyTreeComponent implements OnInit, AfterViewInit {
  @ViewChild('vizContainer', { static: false })
  vizContainer!: ElementRef<HTMLDivElement>;

  isLoading = true;
  error: string | null = null;
  private networkInstance: Network | null = null;

  private visNodes: VisNode[] = [];
  private visEdges: VisEdge[] = [];
  private isViewInitialized = false;
  private isDataReady = false;

  constructor(
    public dialogRef: MatDialogRef<FamilyTreeComponent>,
    @Inject(MAT_DIALOG_DATA) public data: FamilyTreeModalData,
    private neo4jService: Neo4jService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.loadFamilyTree();
  }

  ngAfterViewInit(): void {
    this.isViewInitialized = true;
    this.tryRenderNetwork();
  }

  private async loadFamilyTree(): Promise<void> {
    this.isLoading = true;
    this.error = null;

    const childName = this.data.identity.first_name;
    if (!childName) {
      this.error = "Child's first name is required.";
      this.isLoading = false;
      this.cdr.detectChanges();
      return;
    }

    try {
      const paths: FamilyPath[] = await this.neo4jService.fetchBestFamilyTrees(
        childName,
        this.data.identity.father_name || '',
        this.data.identity.mother_name || ''
      );
      this.processAndRenderPaths(paths || []);
    } catch (err) {
      console.error(err);
      this.error = 'Failed to load family tree.';
      this.visNodes = [];
      this.visEdges = [];
      this.isDataReady = true;
      this.isLoading = false;
      this.cdr.detectChanges();
      this.tryRenderNetwork();
    }
  }

  private processAndRenderPaths(paths: FamilyPath[]): void {
    const nodesMap = new Map<string, VisNode>();
    const edgesList: VisEdge[] = [];
    const central = this.data.identity.first_name;
    const father = this.data.identity.father_name;
    const mother = this.data.identity.mother_name;
    const parentColor = { background: '#C459CB', border: '#A23CA8' };
    const childColor = { background: '#DAE4EF', border: '#666666' };

    // Build nodes
    for (const fp of paths) {
      for (const seg of fp.path?.segments || []) {
        const s = seg.start;
        const e = seg.end;
        const r = seg.relationship;
        const sId = s.identity?.toString();
        const eId = e.identity?.toString();
        const sName = s.properties?.['name'] as string | undefined;
        const eName = e.properties?.['name'] as string | undefined;

        if (sId && sName && !nodesMap.has(sId)) {
          nodesMap.set(sId, {
            id: sId,
            label: sName,
            color: sName === central ? parentColor : childColor,
            shape: 'circle',
          });
        }
        if (eId && eName && !nodesMap.has(eId)) {
          nodesMap.set(eId, {
            id: eId,
            label: eName,
            color: eName === central ? parentColor : childColor,
            shape: 'circle',
          });
        }

        // CHILD_OF edges: push raw then normalize
        if (sId && eId) {
          edgesList.push({ from: sId, to: eId, label: r.type, arrows: r.type === 'MARRIED_TO' ? 'from,to' : 'to' });
        }
      }
    }

    // Normalize CHILD_OF: parents → central, children ← central
    const centralNode = Array.from(nodesMap.values()).find(n => n.label === central);
    if (centralNode) {
      const cid = centralNode.id;
      this.visEdges = edgesList.map(edge => {
        if (edge.label === 'CHILD_OF') {
          // if from is father or mother, that edge should point into central
          if ((father && edge.from === Array.from(nodesMap.values()).find(n => n.label === father)?.id)
            || (mother && edge.from === Array.from(nodesMap.values()).find(n => n.label === mother)?.id)) {
            return { ...edge, from: edge.to, to: edge.from };
          }
          // otherwise if edge.from == central, leave outwards
          if (edge.from === cid) {
            return edge;
          }
          // else if edge.to == cid, flip inward
          if (edge.to === cid) {
            return { ...edge, from: edge.to, to: edge.from };
          }
        }
        return edge;
      });
    } else {
      this.visEdges = edgesList;
    }

    this.visNodes = Array.from(nodesMap.values());
    this.isDataReady = true;
    this.isLoading = false;
    this.cdr.detectChanges();
    this.tryRenderNetwork();
  }

  private tryRenderNetwork(): void {
    if (this.isViewInitialized && this.isDataReady) {
      setTimeout(() => this.renderNetwork(), 0);
    }
  }

  private renderNetwork(): void {
    if (!this.vizContainer?.nativeElement) {
      console.warn('vizContainer not initialized—skipping render.');
      return;
    }
    const container = this.vizContainer.nativeElement;
    const data = {
      nodes: new DataSet<VisNode>(this.visNodes),
      edges: new DataSet<VisEdge>(this.visEdges),
    };
    const options: Options = {
      layout: { hierarchical: { enabled: false } },
      physics: { enabled: true },
      edges: {
        smooth: {
          enabled: true,
          type: 'cubicBezier',
          forceDirection: 'vertical',
          roundness: 0.4,
        },
        font: { size: 12, color: '#888888' },
      },
      nodes: { font: { size: 14, color: '#ffffff' } },
      interaction: { hover: true },
    };

    this.networkInstance?.destroy();
    this.networkInstance = new Network(container, data, options);
  }

  close(): void {
    this.dialogRef.close();
  }
}
