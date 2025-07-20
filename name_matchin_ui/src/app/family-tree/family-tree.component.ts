import { Component, ElementRef, Inject, OnInit, ViewChild, AfterViewInit, ChangeDetectorRef } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { Neo4jService } from '../services/neo4j.service.ts.service'; // Adjusted path
import { FamilyPath } from '../models/family-path/family-path.module';
import { DataSet } from 'vis-data';
import { Network, type Node as VisNode, type Edge as VisEdge } from 'vis-network/standalone'; // vis-network types

// Interface for the data passed to the dialog
export interface FamilyTreeModalData {
  identity: {
    first_name: string;
    father_name?: string; // Optional as per original InputIdentity
    mother_name?: string; // Optional
    // Add other relevant fields from MatchResult.matched_identity if needed by Neo4jService
  };
}

@Component({
  selector: 'app-family-tree',
  standalone: false,
  templateUrl: './family-tree.component.html',
  styleUrls: ['./family-tree.component.css']
})
export class FamilyTreeComponent implements OnInit, AfterViewInit {
  @ViewChild('networkModal', { static: false }) networkModalRef!: ElementRef;
  isLoading = true;
  error: string | null = null;
  private networkInstance: Network | null = null;

  // Properties for managing rendering timing
  private visNodes: VisNode[] = [];
  private visEdges: VisEdge[] = [];
  private isViewInitialized = false;
  private isDataReady = false;

  constructor(
    public dialogRef: MatDialogRef<FamilyTreeComponent>,
    @Inject(MAT_DIALOG_DATA) public data: FamilyTreeModalData,
    private neo4jService: Neo4jService,
    private cdr: ChangeDetectorRef // Injected ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    // Data fetching can start here
    this.loadFamilyTree();
  }

  ngAfterViewInit(): void {
    this.isViewInitialized = true;
    this.tryRenderNetwork();
  }

  async loadFamilyTree(): Promise<void> {
    this.isLoading = true;
    this.error = null;
    if (!this.data || !this.data.identity) {
      this.error = "No identity data provided to the modal.";
      this.isLoading = false;
      this.cdr.detectChanges(); // Detect changes if exiting early
      return;
    }

    const childName = this.data.identity.first_name;
    const fatherName = this.data.identity.father_name || '';
    const motherName = this.data.identity.mother_name || '';

    if (!childName) {
      this.error = "Child's first name is required to fetch the family tree.";
      this.isLoading = false;
      this.cdr.detectChanges(); // Detect changes if exiting early
      return;
    }

    try {
      const paths: FamilyPath[] = await this.neo4jService.fetchBestFamilyTrees(
        childName,
        fatherName,
        motherName
      );

      if (!paths || paths.length === 0) {
        this.error = `No family tree data found for ${childName}.`;
        this.isLoading = false; // Set isLoading before detectChanges
        this.cdr.detectChanges(); // Ensure error message is displayed
        this.processAndRenderPaths([]); // Process empty paths to set flags and potentially clear network
        return;
      }
      this.processAndRenderPaths(paths);
      this.isLoading = false;
      this.cdr.detectChanges(); // Ensure graph container appears if data is loaded
    } catch (err) {
      console.error('Error fetching family tree:', err);
      this.error = `Failed to load family tree. ${err instanceof Error ? err.message : String(err)}`;
      this.isLoading = false;
      this.cdr.detectChanges(); // Ensure error message is displayed
      // Mark data as "ready" for rendering (an empty/error state)
      this.isDataReady = true;
      this.visNodes = [];
      this.visEdges = [];
      this.tryRenderNetwork(); // Attempt to render (which will likely show nothing or an error message via template)
    }
  }

  private processAndRenderPaths(paths: FamilyPath[]): void {
    const nodesMap = new Map<string, VisNode>();
    const edgesList: VisEdge[] = []; // Changed from Set to Array to allow specific edge properties

    const centralNodeName = this.data.identity.first_name;
    const highlightedColor = { background: '#FFD700', border: '#FFA500' }; // Gold/Orange
    const defaultNodeColor = { background: '#DAE4EF', border: '#666666' };

    paths.forEach(fp => {
      if (fp.path && fp.path.segments) {
        fp.path.segments.forEach(seg => {
          const s = seg.start;
          const e = seg.end;
          const r = seg.relationship;

          const sId = s.identity?.toString();
          const eId = e.identity?.toString();
          const sName = s.properties?.['name'] as string | undefined;
          const eName = e.properties?.['name'] as string | undefined;

          if (sId && sName) {
            if (!nodesMap.has(sId)) {
              nodesMap.set(sId, {
                id: sId,
                label: sName,
                color: sName === centralNodeName ? highlightedColor : defaultNodeColor,
                shape: 'box' // Ensure all nodes are boxes
              });
            }
          }
          if (eId && eName) {
            if (!nodesMap.has(eId)) {
              nodesMap.set(eId, {
                id: eId,
                label: eName,
                color: eName === centralNodeName ? highlightedColor : defaultNodeColor,
                shape: 'box' // Ensure all nodes are boxes
              });
            }
          }

          if (sId && eId && r.type) {
            let edge: VisEdge | null = null;
            const relType = r.type;

            // Define edge properties based on relationship type
            if (relType === 'CHILD_OF') {
              // Assuming 's' is child and 'e' is parent from typical Neo4j path segment for CHILD_OF
              // If Neo4j returns (parent)-[:CHILD_OF]->(child), then s=parent, e=child.
              // We need to know the typical query structure.
              // Let's assume for now the query structure implies s is child, e is parent for CHILD_OF path segment.
              // If not, 'from' and 'to' might need swapping based on actual data direction.
              // For vis.js, arrow points from 'from' to 'to'.
              // Child --CHILD_OF--> Parent means from: child, to: parent
              edge = { from: sId, to: eId, label: relType, arrows: 'to' };
            } else if (relType === 'SIBLING_WITH') {
              edge = { from: sId, to: eId, label: relType, arrows: '' }; // No arrows for sibling, or 'from,to'
            } else {
              // Default for other types
              edge = { from: sId, to: eId, label: relType, arrows: 'to' };
            }

            // Check for duplicate edges (e.g. if paths overlap or relationships are bidirectional in query)
            // A more robust way than string key if complex properties matter, but good for basic cases.
            const edgeExists = edgesList.some(exEdge =>
              (exEdge.from === edge?.from && exEdge.to === edge?.to && exEdge.label === edge?.label) ||
              (exEdge.to === edge?.from && exEdge.from === edge?.to && exEdge.label === edge?.label && edge?.arrows === '') // For undirected like SIBLING_WITH
            );

            if (edge && !edgeExists) {
              edgesList.push(edge);
            }
          }
        });
      }
    });

    this.visNodes = Array.from(nodesMap.values());
    this.visEdges = edgesList;

    this.isDataReady = true;
    this.tryRenderNetwork();
  }

  private tryRenderNetwork(): void {
    console.log(
      '[tryRenderNetwork] Called. States:',
      {
        isViewInitialized: this.isViewInitialized,
        isDataReady: this.isDataReady,
        isLoading: this.isLoading,
        error: this.error,
        networkModalRefAvailable: !!(this.networkModalRef && this.networkModalRef.nativeElement)
      }
    );
    if (this.isViewInitialized && this.isDataReady) {
      // Defer rendering to the next tick of the event loop
      console.log('[tryRenderNetwork] Conditions met, scheduling renderNetwork with setTimeout.');
      setTimeout(() => {
        console.log('[tryRenderNetwork] setTimeout callback: Attempting to render.');
        this.renderNetwork(this.visNodes, this.visEdges);
      }, 0);
    } else {
      console.log('[tryRenderNetwork] Conditions not met. View Initialized:', this.isViewInitialized, 'Data Ready:', this.isDataReady);
    }
  }

  private renderNetwork(nodes: VisNode[], edges: VisEdge[]): void {
    console.log('[renderNetwork] Method called.');
    console.log('[renderNetwork] networkModalRef before check:', this.networkModalRef);

    if (!this.networkModalRef || !this.networkModalRef.nativeElement) {
      console.error("Network container is not available. This might be a persistent timing issue or the #networkModal element is missing from the template.");
      return;
    }
    console.log('[renderNetwork] Network container IS available. Proceeding with Vis.js network creation.');
    const container = this.networkModalRef.nativeElement;

    // Ensure nodes have the shape property defined if not set individually
    const processedNodes = nodes.map(node => ({
      ...node,
      shape: node.shape || 'box', // Default to box if not specified
      color: node.color || { background: '#DAE4EF', border: '#666666' } // Default color
    }));

    const data = {
      nodes: new DataSet<VisNode>(processedNodes),
      edges: new DataSet<VisEdge>(edges),
    };

    const options = {
      layout: {
        hierarchical: {
          enabled: true,
          sortMethod: 'directed',
          shakeTowards: 'roots',
          direction: 'UD',
        },
      },
      physics: {
        enabled: true,
        hierarchicalRepulsion: {
          centralGravity: 0.0,
          springLength: 100,
          springConstant: 0.01,
          nodeDistance: 150, // Increased distance a bit
          damping: 0.09
        },
        solver: 'hierarchicalRepulsion'
      },
      nodes: {
        // Global node options (can be overridden by individual node properties)
        shape: 'box', // Default shape
        margin: {
          top: 10,
          right: 10,
          bottom: 10,
          left: 10
        },
        font: {
          face: 'Tahoma',
          color: '#333333',
          size: 14
        },
        // Default color is now set per-node or in processedNodes map
        // color: {
        //   border: '#666666',
        //   background: '#DAE4EF',
        //   highlight: {
        //     border: '#2B7CE9',
        //     background: '#EBF0F5'
        //   }
        // }
      },
      edges: {
        // Global edge options (can be overridden by individual edge properties)
        arrows: 'to', // Default arrows
        smooth: {
          enabled: true,
          type: 'cubicBezier',
          forceDirection: 'vertical',
          roundness: 0.4
        },
        font: {
          align: 'middle',
          size: 12,
          color: '#555555'
        },
        color: { // Default edge color
          color: '#848484',
          highlight: '#2B7CE9',
          hover: '#2B7CE9'
          // inherit: 'from', // Can be useful but let's keep it simple
          // opacity: 1.0
        }
      },
      interaction: {
        hover: true,
        tooltipDelay: 200
      }
    };

    if (this.networkInstance) {
      this.networkInstance.destroy();
    }
    this.networkInstance = new Network(container, data, options);
  }

  close(): void {
    this.dialogRef.close();
  }
}
