import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { FormsModule }   from '@angular/forms';    // ← import here
import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { HttpClientModule } from '@angular/common/http';
import { ReactiveFormsModule } from '@angular/forms';
import { IdentityMatchComponent } from './identity-match/identity-match.component';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatButtonModule } from '@angular/material/button';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';
import {MatCard} from '@angular/material/card';
import {MatIcon} from '@angular/material/icon';
import {MatProgressSpinner} from '@angular/material/progress-spinner';
import { FamilyTreeComponent } from './family-tree/family-tree.component';
import {NgOptimizedImage} from '@angular/common';
import {MatDialogActions, MatDialogContent, MatDialogTitle} from '@angular/material/dialog';


@NgModule({
  declarations: [
    AppComponent,
    IdentityMatchComponent,
    FamilyTreeComponent

  ],
  imports: [
    BrowserModule,
    FormsModule,
    AppRoutingModule,
    HttpClientModule,
    ReactiveFormsModule,
    BrowserModule,
    BrowserAnimationsModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatButtonModule,
    MatCard,
    MatIcon,
    MatProgressSpinner,
    NgOptimizedImage,
    MatDialogContent,
    MatDialogActions,
    MatDialogTitle,
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
