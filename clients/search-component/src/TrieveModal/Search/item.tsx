import { Chunk, ChunkWithHighlights } from "../../utils/types";
import React, { useRef, useState } from "react";
import { ArrowIcon } from "../icons";
import { useModalState } from "../../utils/hooks/modal-context";
import { sendCtrData } from "../../utils/trieve";

type Props = {
  item: ChunkWithHighlights;
  requestID: string;
  index: number;
  className?: string;
};

export const Item = ({ item, requestID, index, className }: Props) => {
  const { props, trieveSDK } = useModalState();
  const Component = item.chunk.link ? "a" : "button";
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const itemRef = useRef<HTMLButtonElement | HTMLLinkElement | any>(null);
  const [isHovered, setIsHovered] = useState(false);

  let descriptionHtml = item.highlights
    ? item.highlights.join("...")
    : item.chunk.chunk_html || "";
  const $descriptionHtml = document.createElement("div");
  $descriptionHtml.innerHTML = descriptionHtml;
  $descriptionHtml.querySelectorAll("b").forEach((b) => {
    return b.replaceWith(b.textContent || "");
  });
  descriptionHtml = $descriptionHtml.innerHTML;

  const openapiRequestHtml = document.createElement("div");
  openapiRequestHtml.innerHTML = item.chunk.chunk_html || "";
  const openapiRequestVerb =
    openapiRequestHtml.querySelector(".openapi-method")?.textContent;

  const chunkHtmlHeadingsDiv = document.createElement("div");
  chunkHtmlHeadingsDiv.innerHTML = item.chunk.chunk_html || "";
  const chunkHtmlHeadings = chunkHtmlHeadingsDiv.querySelectorAll(
    "h1, h2, h3, h4, h5, h6"
  );

  const $firstHeading = chunkHtmlHeadings[0] ?? document.createElement("h1");
  $firstHeading?.querySelectorAll(":not(mark)")?.forEach((tag) => {
    return tag.replaceWith(tag.textContent || "");
  });
  const firstHeadingIdDiv = document.createElement("div");
  firstHeadingIdDiv.innerHTML = $firstHeading.id;
  const firstHeadingId = firstHeadingIdDiv.textContent;
  const cleanFirstHeading = $firstHeading?.innerHTML;

  const titleInnerText = $firstHeading.textContent || "";

  descriptionHtml = descriptionHtml
    .replace(" </mark>", "</mark> ")
    .replace(cleanFirstHeading || "", "");

  for (const heading of chunkHtmlHeadings) {
    const curHeadingText = heading.textContent;

    descriptionHtml = descriptionHtml.replace(curHeadingText || "", "");
  }
  descriptionHtml = descriptionHtml.replace(/([.,!?;:])/g, "$1 ");
  const [shownImage, setShownImage] = useState<string>(
    item.chunk?.image_urls?.[0] || ""
  );
  const price = item.chunk.num_value
    ? ` - ${props.currencyPosition === "before" ? props.defaultCurrency : ""}${
        item.chunk.num_value
      }${props.currencyPosition === "after" ? props.defaultCurrency : ""}`
    : "";
  let title = `${
    cleanFirstHeading ||
    item.chunk.metadata?.title ||
    item.chunk.metadata?.page_title ||
    item.chunk.metadata?.name
  }${price ? "  " + price : ""}`.replace("#", "");

  if (!title.trim() || title == "undefined") {
    return null;
  }

  switch (openapiRequestVerb) {
    case "POST":
      title = title.replace("POST", '<span class="post-method">POST</span>');
      break;
    case "GET":
      title = title.replace("GET", '<span class="get-method">GET</span>');
      break;
    case "PUT":
      title = title.replace("PUT", '<span class="put-method">PUT</span>');
      break;
    case "DELETE":
      title = title.replace(
        "DELETE",
        '<span class="delete-method">DELETE</span>'
      );
      break;
    case "PATCH":
      title = title.replace("PATCH", '<span class="patch-method">PATCH</span>');
      break;
    default:
      break;
  }

  const getChunkPath = () => {
    const urlElements = item.chunk.link?.split("/").slice(3) ?? [];
    if (urlElements?.length > 1) {
      return urlElements
        .slice(0, -1)
        .map((word) => word.replace(/-/g, " "))
        .concat(
          item.chunk.metadata?.title ||
            item.chunk.metadata.summary ||
            urlElements.slice(-1)[0]
        )
        .map((word) =>
          word
            .split(" ")
            .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
            .join(" ")
        )
        .join(" > ");
    } else {
      return item.chunk.metadata?.title ? item.chunk.metadata.title : "";
    }
  };

  const onResultClick = async (
    chunk: Chunk & { position: number },
    requestID: string
  ) => {
    if (props.onResultClick) {
      props.onResultClick(chunk);
    }

    if (props.analytics) {
      await sendCtrData({
        trieve: trieveSDK,
        index: chunk.position,
        requestID: requestID,
        chunkID: chunk.id,
      });
    }
  };

  const linkSuffix = firstHeadingId
    ? `#${firstHeadingId}`
    : `#:~:text=${encodeURIComponent(titleInnerText)}`;

  return (
    <li key={item.chunk.id}>
      <Component
        ref={itemRef}
        id={`trieve-search-item-${index + 1}`}
        className={className ?? "item"}
        onClick={() =>
          onResultClick(
            {
              ...item.chunk,
              position: index,
            },
            requestID
          )
        }
        onMouseEnter={() => {
          setIsHovered(true);
        }}
        onMouseLeave={() => {
          setIsHovered(false);
        }}
        {...(item.chunk.link
          ? {
              href: `${
                item.chunk.link.endsWith("/")
                  ? item.chunk.link.slice(0, -1)
                  : item.chunk.link
              }${linkSuffix}`,
            }
          : {})}
      >
        <div>
          {props.type === "ecommerce" ? (
            item.chunk.image_urls?.length && item.chunk.image_urls[0] ? (
              <img src={shownImage} className="ecommerce-featured-image" />
            ) : (
              <div className="ecommerce-featured-image">
                {props.brandLogoImgSrcUrl ? (
                  <img src={props.brandLogoImgSrcUrl} />
                ) : null}
              </div>
            )
          ) : null}
          {title ? (
            <div>
              {props.type === "docs" ? (
                <h6 className="chunk-path">{getChunkPath()}</h6>
              ) : null}
              <h4
                className="chunk-title"
                dangerouslySetInnerHTML={{
                  __html: title,
                }}
              />
              <p
                className="description"
                dangerouslySetInnerHTML={{
                  __html: descriptionHtml,
                }}
              />
              {props.type === "ecommerce" ? (
                <>
                  {item.chunk.metadata?.variants?.length > 1 ? (
                    <div className="variants">
                      <span className="variants-title">Variants:</span>
                      {(
                        item.chunk.metadata.variants as unknown as {
                          featured_image: { src: string };
                          title: string;
                        }[]
                      )?.map((variant) => (
                        <button
                          onClick={(ev) => {
                            ev.preventDefault();
                            ev.stopPropagation();
                            ev.nativeEvent.stopImmediatePropagation();
                            if (variant.featured_image?.src) {
                              setShownImage(variant.featured_image?.src);
                            }
                          }}
                        >
                          {variant.title}
                        </button>
                      ))}
                    </div>
                  ) : null}
                </>
              ) : null}
            </div>
          ) : (
            <p
              dangerouslySetInnerHTML={{
                __html: descriptionHtml,
              }}
            />
          )}
          <ArrowIcon className={!isHovered ? "text-transparent" : ""} />
        </div>
      </Component>
    </li>
  );
};
