import { Injectable } from '@angular/core';
import neo4j, { Driver, Session, QueryResult } from 'neo4j-driver';
import { environment } from '../../environments/environment';

// Define a new interface for a clean graph structure
export interface FamilyGraph {
  candidate: any; // The main person for this graph
  nodes: any[];
  relationships: any[];
}

@Injectable({ providedIn: 'root' })
export class Neo4jService {
  private driver: Driver;

  constructor() {
    this.driver = neo4j.driver(
      environment.neo4j.boltUrl,
      neo4j.auth.basic(environment.neo4j.user, environment.neo4j.pass)
    );
  }

  // The function now returns a Promise of FamilyGraph[]
  async fetchBestFamilyTrees(
    childName: string,
    fatherName: string,
    motherName: string
  ): Promise<FamilyGraph[]> {
    const session: Session = this.driver.session();

    // This query has been rewritten to enforce logical family rules.
    const cypher = `
      // Part 1: Find the top 3 candidate persons based on the score.
      WITH $childName AS childName, $fatherName AS fatherName, $motherName AS motherName
      MATCH (c:Person {name: childName})
      OPTIONAL MATCH (c)-[:CHILD_OF]->(f:Person {gender: 'ذكر'})
      OPTIONAL MATCH (c)-[:CHILD_OF]->(m:Person {gender: 'أنثى'})
      WITH c,
        (CASE WHEN f.name = fatherName THEN 1 ELSE 0 END
       + CASE WHEN m.name = motherName THEN 1 ELSE 0 END) AS score
      ORDER BY score DESC
      LIMIT 3 // Get the top 3 scoring candidates.

      // Part 2: Use a subquery to process each candidate independently.
      CALL {
          WITH c
          // Step A: Explicitly find a realistic immediate family.
          // Get one father and one mother based on gender.
          OPTIONAL MATCH (c)-[:CHILD_OF]->(father:Person {gender: 'ذكر'})
          OPTIONAL MATCH (c)-[:CHILD_OF]->(mother:Person {gender: 'أنثى'})
          // Get only the first spouse found to keep the graph clean.
          OPTIONAL MATCH (c)-[:MARRIED_TO]-(spouse:Person)
          WITH c, father, mother, spouse LIMIT 1
          // Get all children of the candidate.
          OPTIONAL MATCH (child:Person)-[:CHILD_OF]->(c)

          // Step B: Create a clean list of these specific relatives plus the candidate.
          // This trick collects all non-null family members into a single list.
          WITH c, COLLECT(DISTINCT c) + COLLECT(DISTINCT father) + COLLECT(DISTINCT mother) + COLLECT(DISTINCT spouse) + COLLECT(DISTINCT child) as familyMembers

          // Step C: Collect the nodes and relationships that exist ONLY within this logical, immediate family group.
          UNWIND familyMembers AS person
          WITH c, familyMembers, COLLECT(DISTINCT person) AS nodes

          UNWIND familyMembers AS p1
          OPTIONAL MATCH (p1)-[r:CHILD_OF|MARRIED_TO]-(p2)
          WHERE p2 IN familyMembers AND id(p1) < id(p2)
          WITH c, nodes, COLLECT(DISTINCT r) AS relationships

          // Return the clean graph components for this candidate.
          RETURN c AS candidate, nodes, relationships
      }

      // Part 3: Return the results for each of the top 3 candidates.
      RETURN candidate, nodes, relationships
    `;

    console.log("Executing LOGICAL subgraph query for:", { childName, fatherName, motherName });
    const result: QueryResult = await session.run(cypher, { childName, fatherName, motherName });
    await session.close();
    console.log("Query returned graphs:", result.records.length);

    // Map the Neo4j records to our clean FamilyGraph structure.
    return result.records.map(rec => {
      // Filter out any null nodes or relationships that may have been introduced by OPTIONAL MATCH
      const nodes = rec.get('nodes').filter(Boolean);
      const relationships = rec.get('relationships').filter(Boolean);

      return {
        candidate: rec.get('candidate').properties,
        nodes: nodes.map((node: any) => ({...node.properties, id: node.elementId})),
        relationships: relationships.map((rel: any) => ({...rel.properties, id: rel.elementId, type: rel.type, from: rel.startNodeElementId, to: rel.endNodeElementId}))
      };
    });
  }
}
