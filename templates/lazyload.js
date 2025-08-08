document.addEventListener("DOMContentLoaded", function () {
  const observer = new IntersectionObserver(
    (entries, observer) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          // When the placeholder comes into view
          const placeholder = entry.target;
          const img = placeholder.querySelector("img");
          const thumbnailUrl = placeholder.getAttribute("data-thumbnail-url");
          // Load the actual thumbnail
          img.src = thumbnailUrl;
          img.onload = () => {
            placeholder.classList.remove("thumbnail-placeholder");
          };
          observer.unobserve(placeholder);
        }
      });
    },
    {
      rootMargin: "200px 0px", // Start loading before the image comes into view
      threshold: 0.01, // Trigger as soon as 1% of the image is visible
    }
  );

  document.querySelectorAll(".thumbnail-placeholder").forEach((placeholder) => {
    observer.observe(placeholder);
  });
});
