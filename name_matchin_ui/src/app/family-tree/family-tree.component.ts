import { Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { FamilyPath } from '../models/family-path/family-path.module';
import { Neo4jService } from '../services/neo4j.service.ts.service';
import type { Node as NeoNode, Relationship as NeoRel } from 'neo4j-driver';
import { DataSet } from 'vis-data';
// vis-network standalone gives you Network, Node, Edge, etc.
import { Network, type Node, type Edge } from 'vis-network/standalone';
@Component({
  selector: 'app-family-tree',
  standalone: false,
  templateUrl: './family-tree.component.html',
  styleUrls: ['./family-tree.component.css']
})
export class FamilyTreeComponent implements OnInit {
  @ViewChild('network', { static: true }) networkRef!: ElementRef;
  childName = '';
  fatherName = '';
  motherName = '';

  constructor(private neo4j: Neo4jService) {}

  ngOnInit() {}

  async search(): Promise<void> {
    // Fetch the top-3 matches with their mini-trees
    const paths: FamilyPath[] = await this.neo4j.fetchBestFamilyTrees(
      this.childName,
      this.fatherName,
      this.motherName
    );

    // Build unique node & edge lists
    const nodesMap = new Map<string, Node>();
    const edgesMap = new Set<string>();

    paths.forEach(fp => {
      fp.path.segments.forEach(seg => {
        const s = seg.start;
        const e = seg.end;
        const r = seg.relationship;
        const sid = s.identity.toString();
        const eid = e.identity.toString();

        // Use bracket notation per TS index signature
        nodesMap.set(sid, { id: sid, label: s.properties['name'] });
        nodesMap.set(eid, { id: eid, label: e.properties['name'] });

        const edgeKey = `${sid}-${eid}-${r.type}`;
        if (!edgesMap.has(edgeKey)) {
          edgesMap.add(edgeKey);
        }
      });
    });

    // Convert to vis DataSets
    const visNodes = new DataSet(Array.from(nodesMap.values()));
    const visEdges = new DataSet(
      Array.from(edgesMap).map(key => {
        const [from, to, type] = key.split('-');
        return { from, to, label: type } as Edge;
      })
    );

    // Render the network
    new Network(
      this.networkRef.nativeElement,
      { nodes: visNodes, edges: visEdges },
      {
        layout: { hierarchical: true },
        physics: { enabled: true }
      }
    );
  }
}
