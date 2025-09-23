import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { IdentityMatchComponent } from './identity-match/identity-match.component';
import { LoginComponent } from './login/login.component';
import { RegisterComponent } from './register/register.component';
import { AuthGuard } from './services/auth.guard';
import { UserManagementComponent } from './user-management/user-management.component';

const routes: Routes = [
  { path: '', redirectTo: '/identity-match', pathMatch: 'full' },
  { path: 'login', component: LoginComponent },
  { path: 'register', component: RegisterComponent },
  { path: 'identity-match', component: IdentityMatchComponent, canActivate: [AuthGuard] },
  { path: 'user-management', component: UserManagementComponent, canActivate: [AuthGuard] },
  { path: '**', redirectTo: '/identity-match' }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
