import { BrowserModule } from "@angular/platform-browser";
import { HttpClientModule } from "@angular/common/http";
import { NgModule } from "@angular/core";

import { AppComponent } from "./app.component";
import { AuthServiceConfig } from "./auth/auth.service";
import { GoogleAuthProvider } from "./auth/provider/google-auth-provider";

const config = new AuthServiceConfig([GoogleAuthProvider]);
export function provideConfig() {
  return config;
}

@NgModule({
  declarations: [AppComponent],
  imports: [BrowserModule, HttpClientModule],
  providers: [
    {
      provide: AuthServiceConfig,
      useFactory: provideConfig
    },
    {
      provide: Window,
      useValue: window
    },
    GoogleAuthProvider
  ],
  bootstrap: [AppComponent]
})
export class AppModule {}
