<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="css/bootstrap.min.css" rel="stylesheet">
    <link href="css/style.css" rel="stylesheet">
    <title>{{title}}</title>
  </head>
  <body>
    <main>

      <section class="py-1 text-center container">
        <div class="row py-lg-3">
          <div class="col-lg-6 col-md-8 mx-auto">
            <h1 class="fw-light">{{title}}</h1>
          </div>
        </div>
      </section>

      <div class="album py-5 bg-light">
        <div class="container">
          <div class="row row-cols-1 row-cols-sm-2 row-cols-md-3 row-cols-lg-4 gx-3 gy-5">
            {{#each image_groups}}
            {{#each images}}
            <div class="col">
              <h2>
              </h2>
              <div class="card shadow-sm">
                <a href="{{../url}}/{{file_name}}"><img class="card-img-top" src="{{thumbnail}}"></a>
                <div class="card-body">
                  {{#if ../markdown_content}}
                  <a href="{{../url}}/index.html">{{../title}} ↗</a>
                  {{else}}
                  {{../title}}
                  {{/if}}
                  <div class="d-flex justify-content-between align-items-center">
                    <small class="text-muted">{{name}}</small>
                    <small class="text-muted">{{../date}}</small>
                  </div>
                </div>
              </div>
            </div>
            {{/each}}
            {{/each}}
          </div>
        </div>
      </div>

    </main>

    {{#if footer}}
    <footer class="bd-footer text-muted bg-light">
      <div class="container-fluid p-5">
        <div class="row justify-content-center">
          <div class="col-auto">
            {{{footer}}}
          </div>
        </div>
      </div>
    </footer>
    {{/if}}

    <script src="js/bootstrap.bundle.min.js"></script>
  </body>
</html>