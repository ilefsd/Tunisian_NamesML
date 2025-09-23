import {
  Component,
  OnInit,
  Inject,
  ViewChildren,
  QueryList,
  ElementRef,
  ChangeDetectorRef,
  AfterViewInit,
} from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
// Import the new interface and the service
import { Neo4jService, FamilyGraph } from '../services/neo4j.service.ts.service';
import { DataSet } from 'vis-data';
import {
  Network,
  type Node as VisNode,
  type Edge as VisEdge,
  Options,
} from 'vis-network/standalone';
// Imports for the new image generation feature
import { DomSanitizer, SafeUrl } from '@angular/platform-browser';
import { FamilyTreeImageService } from '../services/family-tree-image.service';


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
  @ViewChildren('vizContainer') vizContainers!: QueryList<ElementRef<HTMLDivElement>>;

  isLoading = true; // Re-used for both graph loading and image generation
  error: string | null = null;
  familyGraphs: FamilyGraph[] = [];
  // New property to hold the formatted JSON string for display
  rawJsonForDisplay: string | null = null;
  private networkInstances: Network[] = [];

  // Properties for the new image generation feature
  imageUrl: SafeUrl | null = null;

  constructor(
    public dialogRef: MatDialogRef<FamilyTreeComponent>,
    @Inject(MAT_DIALOG_DATA) public data: FamilyTreeModalData,
    private neo4jService: Neo4jService,
    private cdr: ChangeDetectorRef,
    // Injections for the new image generation feature
    private familyTreeService: FamilyTreeImageService,
    private sanitizer: DomSanitizer
  ) {}

  ngOnInit(): void {
    this.loadAndBuild();
  }

  ngAfterViewInit(): void {
    this.vizContainers.changes.subscribe(() => {
      this.renderAllNetworks();
    });
  }

  private async loadAndBuild(): Promise<void> {
    this.isLoading = true;
    this.error = null;
    this.rawJsonForDisplay = null; // Reset on new load
    this.imageUrl = null; // Also reset the image URL
    this.cdr.detectChanges();

    try {
      const name = this.data.identity.first_name;
      if (!name) throw new Error("Child's first name is required.");

      this.familyGraphs =
        (await this.neo4jService.fetchBestFamilyTrees(
          name,
          this.data.identity.father_name || '',
          this.data.identity.mother_name || ''
        )) || [];

      if (this.familyGraphs.length === 0) {
        this.error = "No family data found for the given names.";
      } else {
        // Convert the received graph data to a formatted JSON string
        this.rawJsonForDisplay = JSON.stringify(this.familyGraphs, null, 2);
      }

    } catch (err: any) {
      console.error(err);
      this.error = err.message || 'Failed to load family data.';
    } finally {
      this.isLoading = false;
      this.cdr.detectChanges();
    }
  }

  // New method for generating the family tree image
  generateImage(): void {
    if (this.familyGraphs.length === 0) {
      this.error = "Cannot generate image, no family data is loaded.";
      return;
    }

    // Use the first available graph data to send to the webhook
    const neo4jData = this.familyGraphs[0];

    this.isLoading = true;
    this.imageUrl = null;

    this.familyTreeService.generateTreeImage(neo4jData)
      .subscribe({
        next: (blob) => {
          console.log('Received blob from API:', blob); // <-- Debugging log
          const objectUrl = URL.createObjectURL(blob);
          this.imageUrl = this.sanitizer.bypassSecurityTrustUrl(objectUrl);
          console.log('Generated image URL:', this.imageUrl); // <-- Debugging log
          this.isLoading = false;
        },
        error: (err) => {
          console.error('Error generating image:', err);
          this.error = 'Failed to generate family tree image.';
          this.isLoading = false;
        }
      });
  }


  private renderAllNetworks(): void {
    this.networkInstances.forEach(instance => instance.destroy());
    this.networkInstances = [];

    const containers = this.vizContainers.toArray();

    this.familyGraphs.forEach((graphData, index) => {
      const container = containers[index]?.nativeElement;
      if (!container) return;

      const visNodes: VisNode[] = graphData.nodes.map(node => {
        const visNode: VisNode = {
          id: node.id,
          label: node.name,
        };
        // Highlight the main candidate and pin them to the center to create the radial layout
        if (node.name === graphData.candidate.name && node.family === graphData.candidate.family) {
          visNode.color = { background: '#C459CB', border: '#A23CA8', highlight: { background: '#D980DE', border: '#A23CA8'} };
          visNode.font = { color: '#fff', size: 16 };
          visNode.shape = 'circle';
          visNode.fixed = { x: true, y: true }; // Pin the node to the center
          visNode.x = 0;
          visNode.y = 0;
        }
        return visNode;
      });

      const visEdges: VisEdge[] = graphData.relationships.map(rel => {
        const edge: VisEdge = {
          from: rel.from,
          to: rel.to,
          label: rel.type.replace('_WITH', '').replace('_TO','').replace('_OF', ''),
        };
        switch (rel.type) {
          case 'MARRIED_TO':
            edge.color = { color: '#e04141' };
            edge.dashes = true;
            break;
          case 'SIBLING_WITH':
            edge.color = { color: '#3c78d8' };
            break;
          default: // CHILD_OF
            edge.color = { color: '#848484' };
            edge.arrows = 'to';
            break;
        }
        return edge;
      });

      const data = {
        nodes: new DataSet<VisNode>(visNodes),
        edges: new DataSet<VisEdge>(visEdges),
      };

      // --- CORRECTED FORCE-DIRECTED LAYOUT OPTIONS ---
      const options: Options = {
        // Use the BarnesHut physics model for a more natural, spread-out graph
        physics: {
          barnesHut: {
            gravitationalConstant: -8000, // Pushes nodes away from each other
            centralGravity: 0.15,         // Pulls the whole graph to the center
            springLength: 250,            // The ideal length of an edge
            springConstant: 0.05,
            damping: 0.09,
          },
          stabilization: {
            iterations: 400, // More iterations for a more stable layout
          }
        },
        interaction: {
          dragNodes: true,
          zoomView: true,
          navigationButtons: true,
        },
        edges: {

          font: {
            size: 9,
            color: '#666',
            strokeWidth: 0,
            align: 'top',
          },
        },
        nodes: {
          // Use a circle shape like the Neo4j browser
          shape: 'circle',
          font: {
            size: 14,
            color: '#333'
          },
          borderWidth: 2,
        },
      };

      this.networkInstances.push(new Network(container, data, options));
    });
  }

  close(): void {
    this.dialogRef.close();
  }
}
