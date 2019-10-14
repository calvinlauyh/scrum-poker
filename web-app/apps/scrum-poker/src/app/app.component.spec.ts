import { TestBed, async } from "@angular/core/testing";
import { AppComponent } from "./app.component";
import { AuthServiceConfig } from "./auth/auth.service";
import { HttpClientModule } from '@angular/common/http';

describe("AppComponent", () => {
  beforeEach(async(() => {
    const window: Readonly<any> = {
      localStorage: {
        setItem: () => {},
        getItem: () => {},
        removeItem: () => {}
      }
    };
    TestBed.configureTestingModule({
      declarations: [AppComponent],
      imports: [HttpClientModule],
      providers: [
        {
          provide: AuthServiceConfig,
          useValue: new AuthServiceConfig([])
        },
        {
          provide: Window,
          useValue: window
        }
      ]
    }).compileComponents();
  }));

  it("should create the app", () => {
    const fixture = TestBed.createComponent(AppComponent);
    const app = fixture.debugElement.componentInstance;
    expect(app).toBeTruthy();
  });

  it(`should have as title 'Scrum Poker'`, () => {
    const fixture = TestBed.createComponent(AppComponent);
    const app = fixture.debugElement.componentInstance;
    expect(app.title).toEqual("Scrum Poker");
  });

  it("should render title in a h1 tag", () => {
    const fixture = TestBed.createComponent(AppComponent);
    fixture.detectChanges();
    const compiled = fixture.debugElement.nativeElement;
    expect(compiled.querySelector("h1").textContent).toContain(
      "Welcome to Scrum Poker!"
    );
  });
});
